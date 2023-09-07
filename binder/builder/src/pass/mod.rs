use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use structopt::StructOpt;

use libra_shared::config::PATH_ROOT;
use libra_shared::dep::Dependency;

use crate::deps::artifact_for_llvm;
use crate::deps::llvm::DepLLVM;

// path constants
static SEGMENTS: [&str; 1] = ["oracle"];

#[derive(StructOpt)]
pub struct PassArgs {
    /// Version of the LLVM dependency
    #[structopt(short, long)]
    llvm_version: Option<String>,

    /// Force the build to proceed
    #[structopt(short, long)]
    force: bool,
}

impl PassArgs {
    pub fn build(self, studio: &Path) -> Result<()> {
        let Self {
            llvm_version,
            force,
        } = self;

        // derive deps and paths
        let (config_hash, dep_llvm) = derive_deps(studio, llvm_version.as_deref())?;
        let resolver_llvm = DepLLVM::artifact_resolver(&dep_llvm);

        let mut path_src = PATH_ROOT.clone();
        path_src.extend(SEGMENTS);

        let mut path_build = studio.to_path_buf();
        path_build.extend(SEGMENTS);
        path_build.push(config_hash);

        // clean out previous build if needed
        if path_build.exists() {
            if !force {
                bail!(
                    "Build directory {} already exists",
                    path_build.to_str().unwrap()
                );
            }
            fs::remove_dir_all(&path_build)?;
        }
        fs::create_dir_all(&path_build)?;

        // configure
        let mut cmd = Command::new("cmake");
        cmd.arg("-G")
            .arg("Ninja")
            .arg(format!(
                "-DCFG_LLVM_INSTALL_DIR={}",
                resolver_llvm
                    .path_install()
                    .to_str()
                    .ok_or_else(|| anyhow!("non-ascii path"))?
            ))
            .arg("-DCMAKE_BUILD_TYPE=Debug")
            .arg(path_src);
        cmd.current_dir(&path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }

        // build
        let mut cmd = Command::new("cmake");
        cmd.arg("--build").arg(&path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Build failed"));
        }

        // done
        Ok(())
    }
}

/// Derive the config hash for the pass
fn derive_deps(studio: &Path, llvm_version: Option<&str>) -> Result<(String, PathBuf)> {
    // get dep: llvm
    let path_llvm = artifact_for_llvm(studio, llvm_version)?;
    let repr_llvm = path_llvm
        .to_str()
        .ok_or_else(|| anyhow!("non-ascii path"))?;

    // config hash
    let mut hasher = DefaultHasher::new();
    repr_llvm.hash(&mut hasher);
    let config_hash = hasher.finish();

    // done
    Ok((format!("{:#18x}", config_hash), path_llvm))
}

/// Retrieve the artifact path
pub fn artifact(studio: &Path, llvm_version: Option<&str>) -> Result<PathBuf> {
    let (config_hash, _) = derive_deps(studio, llvm_version)?;
    let mut path_build = studio.to_path_buf();
    path_build.extend(SEGMENTS);
    path_build.extend([config_hash.as_str(), "Libra", "libLibra.so"]);
    Ok(path_build)
}
