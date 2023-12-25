use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use anyhow::{anyhow, bail, Result};
use log::debug;

use libra_builder::ResolverLLVM;
use libra_engine::error::{EngineError, EngineResult};
use libra_engine::flow::fixedpoint::FlowFixedpoint;
use libra_engine::flow::shared::Context;
use libra_shared::compile_db::{
    ClangCommand, ClangSupportedLanguage, CompileDB, CompileEntry, TokenStream,
};
use libra_shared::dep::{DepState, Dependency, Resolver};
use libra_shared::git::GitRepo;

use crate::common::{TestCase, TestSuite};

static PATH_REPO: [&str; 2] = ["deps", "llvm-test-suite"];
static PATH_WORKSPACE: [&str; 2] = ["testsuite", "external"];

/// Maximum number of fixedpoint optimization
static MAX_ROUNDS_OF_FIXEDPOINT_OPTIMIZATION: usize = 16;

/// Test cases to be ignored
static IGNORED_TEST_CASES: [&str; 1] = [
    // deserialization recursion limit exceeded
    "SingleSource/Benchmarks/Misc-C++-EH/spirit.test",
];

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
        "-DTEST_SUITE_SUBDIRS=SingleSource".to_string(),
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

impl TestSuite<TestCaseExternal, ResolverLLVMExternal> for DepLLVMExternal {
    fn wks_path_from_studio() -> &'static [&'static str] {
        PATH_WORKSPACE.as_ref()
    }

    fn discover_test_cases(
        _repo: &GitRepo,
        resolver: &ResolverLLVMExternal,
    ) -> Result<Vec<TestCaseExternal>> {
        let commands = Self::parse_compile_database(resolver)?;
        Self::lit_test_discovery(resolver, commands)
    }
}

impl DepLLVMExternal {
    fn parse_compile_entry(entry: &CompileEntry) -> Result<Option<(String, ClangCommand)>> {
        let workdir = PathBuf::from(&entry.directory);
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
        let cmd = match sub_token {
            "clang" => ClangCommand::new(false, workdir, tokens)?,
            "clang++" => ClangCommand::new(true, workdir, tokens)?,
            _ => bail!("unrecognized compiler"),
        };
        sub_tokens.prev_expect_literal("bin")?;

        // make sure the cmd entry conforms to our expectations
        let outputs = cmd.outputs();
        if outputs.len() != 1 {
            bail!("expect one and only one output");
        }

        // extract the marker (from the output side)
        let path = Path::new(outputs.into_iter().next().unwrap());

        let mut seen_cmakefiles = false;
        let mut seen_output_dir = false;
        let mut path_output_trimmed = PathBuf::new();
        for token in path {
            if token == "CMakeFiles" {
                seen_cmakefiles = true;
                continue;
            }
            let segment = Path::new(token);
            if seen_cmakefiles {
                if segment.extension().map_or(true, |ext| ext != "dir") {
                    bail!("no CMakeFiles/<target>.dir/ in output path");
                }

                seen_output_dir = true;
                path_output_trimmed.push(segment.with_extension("test"));
                break;
            }
            path_output_trimmed.push(segment);
        }
        if !seen_output_dir {
            bail!("no CMakeFiles/<target>.dir/ in output path");
        }

        // return both
        let mark = path_output_trimmed
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("non-ascii path"))?;
        Ok(Some((mark, cmd)))
    }

    fn parse_compile_database(
        resolver: &ResolverLLVMExternal,
    ) -> Result<BTreeMap<String, ClangCommand>> {
        let comp_db = CompileDB::new(&resolver.path_compile_db)?;

        // collect commands into a map
        let mut commands = BTreeMap::new();
        for entry in comp_db.entries {
            let entry_opt = Self::parse_compile_entry(&entry)
                .map_err(|e| anyhow!("failed to parse '{}': {}", entry.command, e))?;
            if let Some((mark, cmd)) = entry_opt {
                match commands.insert(mark, cmd) {
                    None => (),
                    Some(existing) => bail!("same output is produced twice: {}", existing),
                }
            }
        }
        Ok(commands)
    }

    fn lit_test_discovery(
        resolver: &ResolverLLVMExternal,
        mut commands: BTreeMap<String, ClangCommand>,
    ) -> Result<Vec<TestCaseExternal>> {
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
        let mut result = vec![];
        for line in lines {
            let mut tokens = line.trim().split(" :: ");

            let ty = tokens.next().ok_or_else(|| anyhow!("expect test type"))?;
            if ty != "test-suite" {
                bail!("unexpected test type: {}", ty);
            }

            let name = tokens.next().ok_or_else(|| anyhow!("expect test name"))?;
            let command = match commands.remove(name) {
                None => bail!("test not in compile db: {}", name),
                Some(cmd) => cmd,
            };

            // check existence
            let path_test = resolver.path_artifact.join(name);
            if !path_test.exists() {
                bail!("test marker does not exist: {}", name);
            }

            // create the test case
            result.push(TestCaseExternal {
                name: name.to_string(),
                _path: path_test,
                command,
            });
        }

        Ok(result)
    }
}

/// A test case from the llvm-test-suite
pub struct TestCaseExternal {
    name: String,
    _path: PathBuf,
    command: ClangCommand,
}

impl TestCaseExternal {
    /// Run libra engine
    fn libra_workflow(
        ctxt: &Context,
        command: &ClangCommand,
        input: &Path,
        output: &Path,
    ) -> EngineResult<()> {
        // compile
        let bc_init = output.join("init.bc");
        ctxt.compile_to_bitcode(input, &bc_init, command.gen_args_for_libra())
            .map_err(|e| EngineError::CompilationError(format!("Error during clang: {}", e)))?;
        ctxt.disassemble_in_place(&bc_init)
            .map_err(|e| EngineError::CompilationError(format!("Error during disas: {}", e)))?;

        // fixedpoint
        let flow_fp = FlowFixedpoint::new(
            ctxt,
            bc_init,
            output.to_path_buf(),
            Some(MAX_ROUNDS_OF_FIXEDPOINT_OPTIMIZATION),
        );
        flow_fp.execute()?;

        // done with everything
        Ok(())
    }
}

impl TestCase for TestCaseExternal {
    fn name(&self) -> &str {
        &self.name
    }

    fn run_libra(
        &self,
        ctxt: &Context,
        workdir: &Path,
    ) -> Result<(String, Option<EngineResult<()>>)> {
        let Self {
            name,
            _path: _,
            command,
        } = self;

        // TODO: investigate such test cases that should be ignored
        if IGNORED_TEST_CASES.contains(&name.as_str()) {
            return Ok((name.to_string(), None));
        }

        // TODO: support other languages like ObjC
        match command.infer_language() {
            None => bail!("unable to infer input language"),
            Some(lang) => match lang {
                ClangSupportedLanguage::C
                | ClangSupportedLanguage::CPP
                | ClangSupportedLanguage::Bitcode => (),
                _ => return Ok((name.to_string(), None)),
            },
        }

        // retrieve input
        let inputs = command.inputs();
        if inputs.len() != 1 {
            // NOTE: this is true because we use SingleSource tests only
            bail!("expect one and only one input");
        }
        let input = inputs.into_iter().next().unwrap();

        // report progress
        debug!("running test case: {}", name);

        // prepare output directory
        let output_dir = workdir.join(name);
        fs::create_dir_all(&output_dir)?;

        // temporarily change directory
        let cursor = env::current_dir()?;
        env::set_current_dir(&command.workdir)?;

        // workflow
        let result = Self::libra_workflow(ctxt, command, Path::new(input), &output_dir);

        // clean-up
        env::set_current_dir(cursor)?;
        Ok((name.to_string(), Some(result)))
    }
}
