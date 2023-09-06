use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use fs_extra::dir;

use libra_engine::flow::shared::Context;
use libra_shared::compile_db::{CompileDB, CompileEntry, TokenStream};
use libra_shared::dep::Dependency;
use libra_shared::git::GitRepo;

use crate::common::TestSuite;

static PATH_REPO: [&str; 2] = ["deps", "llvm-test-suite"];

fn baseline_cmake_options(path_src: &Path) -> Result<Vec<String>> {
    let ctxt = Context::new();
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
            ["SingleSource", "MultiSource", "Bitcode"].join(";")
        ),
    ])
}

/// Represent the llvm-test-suite
pub struct DepLLVMTestSuite {}

impl Dependency for DepLLVMTestSuite {
    fn repo_path_from_root() -> &'static [&'static str] {
        &PATH_REPO
    }

    fn list_build_options(path_src: &Path, path_build: &Path) -> Result<()> {
        // dump cmake options
        let mut cmd = Command::new("cmake");
        cmd.arg("-LAH")
            .args(baseline_cmake_options(path_src)?)
            .arg(path_src)
            .current_dir(path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }

        // done
        Ok(())
    }

    fn build(path_src: &Path, path_build: &Path, artifact: &Path) -> Result<()> {
        // llvm configuration
        let mut cmd = Command::new("cmake");
        cmd.arg("-G")
            .arg("Ninja")
            .args(baseline_cmake_options(path_src)?)
            .arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON")
            .arg(path_src)
            .current_dir(path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }

        // build
        let mut cmd = Command::new("cmake");
        cmd.arg("--build").arg(path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Build failed"));
        }

        // install
        fs::create_dir_all(artifact)?;
        let options = dir::CopyOptions::new().content_only(true);
        dir::copy(path_build, artifact, &options)?;

        // done
        Ok(())
    }
}

impl TestSuite for DepLLVMTestSuite {
    fn run(_repo: &GitRepo, path_artifact: &Path) -> Result<()> {
        // parse compilation database
        Self::parse_compile_database(path_artifact)?;
        Ok(())
    }
}

impl DepLLVMTestSuite {
    fn parse_compile_entry(entry: &CompileEntry) -> Result<Option<()>> {
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
        match sub_token {
            "clang" => {}
            "clang++" => {}
            _ => bail!("unrecognized compiler"),
        }
        sub_tokens.prev_expect_literal("bin")?;

        Ok(Some(()))
    }

    fn parse_compile_database(path_artifact: &Path) -> Result<()> {
        let comp_db = CompileDB::new(&path_artifact.join("compile_commands.json"))?;
        for entry in comp_db.entries {
            Self::parse_compile_entry(&entry)
                .map_err(|e| anyhow!("failed to parse '{}': {}", entry.command, e))?;
        }
        Ok(())
    }
}
