use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use libra_builder::ResolverLLVM;

use libra_engine::flow::shared::Context;
use libra_shared::compile_db::{ClangCommand, CompileDB, CompileEntry, TokenStream};
use libra_shared::dep::{DepState, Dependency, Resolver};
use libra_shared::git::GitRepo;

use crate::common::TestSuite;

static PATH_REPO: [&str; 2] = ["deps", "llvm-test-suite"];

/// Get baseline cmake command
fn baseline_cmake_options(path_src: &Path) -> Result<Vec<String>> {
    let ctxt = Context::new()?;
    let profile = path_src
        .join("cmake")
        .join("caches")
        .join("Debug.cmake")
        .into_os_string()
        .into_string()
        .map_err(|_| anyhow!("non-ascii path"))?;

    Ok(vec![
        format!("-DCMAKE_C_COMPILER={}", ctxt.path_llvm(["bin", "clang"])?),
        format!("-C{}", profile),
        format!(
            "-DTEST_SUITE_SUBDIRS={}",
            ["SingleSource", "Bitcode"].join(";")
        ),
    ])
}

/// Artifact path resolver for LLVM
pub struct ResolverLLVMExternal {
    /// Base path for the artifact directory
    path_artifact: PathBuf,
    /// <artifact>/compile_commands.json
    path_compile_db: PathBuf,
}

impl Resolver for ResolverLLVMExternal {
    fn construct(path: PathBuf) -> Self {
        Self {
            path_compile_db: path.join("compile_commands.json"),
            path_artifact: path,
        }
    }

    fn destruct(self) -> PathBuf {
        self.path_artifact
    }

    fn seek() -> Result<(GitRepo, Self)> {
        DepState::<ResolverLLVMExternal, DepLLVMExternal>::new()?.into_source_and_artifact()
    }
}

/// Represent the llvm-test-suite
pub struct DepLLVMExternal {}

impl Dependency<ResolverLLVMExternal> for DepLLVMExternal {
    fn repo_path_from_root() -> &'static [&'static str] {
        &PATH_REPO
    }

    fn list_build_options(path_src: &Path, path_config: &Path) -> Result<()> {
        let mut cmd = Command::new("cmake");
        cmd.arg("-LAH")
            .args(baseline_cmake_options(path_src)?)
            .arg(path_src)
            .current_dir(path_config);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }
        Ok(())
    }

    fn build(path_src: &Path, resolver: &ResolverLLVMExternal) -> Result<()> {
        // config
        let mut cmd = Command::new("cmake");
        cmd.arg("-G")
            .arg("Ninja")
            .args(baseline_cmake_options(path_src)?)
            .arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON")
            .arg(path_src)
            .current_dir(&resolver.path_artifact);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }

        // build
        let mut cmd = Command::new("cmake");
        cmd.arg("--build").arg(&resolver.path_artifact);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Build failed"));
        }

        // done
        Ok(())
    }
}

impl TestSuite<ResolverLLVMExternal> for DepLLVMExternal {
    fn run(_repo: GitRepo, resolver: ResolverLLVMExternal) -> Result<()> {
        // parse compilation database
        let commands = Self::parse_compile_database(&resolver)?;

        // lit test discovery
        Self::lit_test_discovery(&resolver, &commands)?;

        Ok(())
    }
}

impl DepLLVMExternal {
    fn parse_compile_entry(entry: &CompileEntry) -> Result<Option<ClangCommand>> {
        let mut tokens = TokenStream::new(entry.command.split(' '));

        // check the header
        let token = tokens.next_expect_token()?;

        let mut sub_tokens = TokenStream::new(token.split('/'));
        let sub_token = sub_tokens.prev_expect_token()?;
        match sub_token {
            "timeit" => {
                sub_tokens.prev_expect_literal("tools")?;
            }
            "clang" | "clang++" => {
                // this is for host compilation, ignore them
                return Ok(None);
            }
            _ => bail!("unrecognized binary"),
        }

        // next token should be summary
        tokens.next_expect_literal("--summary")?;
        let token = tokens.next_expect_token()?;
        if !token.ends_with(".time") {
            bail!("expect a timeit summary file");
        }

        // next token should be a llvm tool
        let token = tokens.next_expect_token()?;

        let mut sub_tokens = TokenStream::new(token.split('/'));
        let sub_token = sub_tokens.prev_expect_token()?;
        let clang_cmd = match sub_token {
            "clang" => ClangCommand::new(false, tokens)?,
            "clang++" => ClangCommand::new(true, tokens)?,
            _ => bail!("unrecognized compiler"),
        };
        sub_tokens.prev_expect_literal("bin")?;

        Ok(Some(clang_cmd))
    }

    fn parse_compile_database(
        resolver: &ResolverLLVMExternal,
    ) -> Result<BTreeMap<String, ClangCommand>> {
        let comp_db = CompileDB::new(&resolver.path_compile_db)?;

        // collect commands into a map
        let mut commands = BTreeMap::new();
        for entry in comp_db.entries {
            let cmd_opt = Self::parse_compile_entry(&entry)
                .map_err(|e| anyhow!("failed to parse '{}': {}", entry.command, e))?;
            if let Some(cmd) = cmd_opt {
                let inputs = cmd.inputs();
                // NOTE: this is true as we test on single-source only
                if inputs.len() != 1 {
                    bail!("expect one and only one input: {}", cmd);
                }
                let input = inputs.into_iter().next().unwrap().to_string();
                match commands.insert(input, cmd) {
                    None => (),
                    Some(existing) => bail!("same input is used in two tests: {}", existing),
                }
            }
        }
        Ok(commands)
    }

    fn lit_test_discovery(
        resolver: &ResolverLLVMExternal,
        commands: &BTreeMap<String, ClangCommand>,
    ) -> Result<()> {
        // locate the lit tool
        let (_, pkg_llvm) = ResolverLLVM::seek()?;
        let bin_lit = pkg_llvm.path_build().join("bin").join("llvm-lit");

        // run discovery
        let output = Command::new(bin_lit)
            .arg("--show-tests")
            .arg(&resolver.path_artifact)
            .output()?;

        // sanity check the execution
        if !output.stderr.is_empty() {
            bail!(
                "stderr: {}",
                String::from_utf8(output.stderr)
                    .unwrap_or_else(|_| "<unable-to-parse>".to_string())
            );
        }
        if !output.status.success() {
            bail!("lit test discovery fails");
        }

        let content = String::from_utf8(output.stdout)?;
        let mut lines = content.lines();

        // skip first line
        if lines.next().map_or(true, |l| l != "-- Available Tests --") {
            bail!("invalid header line");
        }

        // parse the result
        for line in lines {
            let mut tokens = line.trim().split(" :: ");
            let ty = tokens.next().ok_or_else(|| anyhow!("expect test type"))?;
            if ty != "test-suite" {
                bail!("unexpected test type: {}", ty);
            }
            let name = tokens.next().ok_or_else(|| anyhow!("expect test name"))?;

            // check existence
            let path_test = resolver.path_artifact.join(name);
            if !path_test.exists() {
                bail!("test marker does not exist: {}", name);
            }

            // TODO
            println!("{}", name);
        }

        Ok(())
    }
}
