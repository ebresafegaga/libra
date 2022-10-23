use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Result};
use structopt::StructOpt;

use libra_shared::config::PATH_ROOT;

use crate::deps::artifact_for_llvm;

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
            .arg(format!("-DCFG_LLVM_INSTALL_DIR={}", dep_llvm))
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
fn derive_deps(studio: &Path, llvm_version: Option<&str>) -> Result<(String, String)> {
    // get dep: llvm
    let dep_llvm = artifact_for_llvm(studio, llvm_version)?
        .into_os_string()
        .into_string()
        .unwrap();

    // config hash
    let mut hasher = DefaultHasher::new();
    dep_llvm.hash(&mut hasher);
    let config_hash = hasher.finish();

    // done
    Ok((format!("{:#18x}", config_hash), dep_llvm))
}

/// Retrieve the artifact path
pub fn artifact(studio: &Path, llvm_version: Option<&str>) -> Result<PathBuf> {
    let (config_hash, _) = derive_deps(studio, llvm_version)?;
    let mut path_build = studio.to_path_buf();
    path_build.extend(SEGMENTS);
    path_build.extend([config_hash.as_str(), "Libra", "libLibra.so"]);
    Ok(path_build)
}