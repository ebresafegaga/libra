use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use libra_shared::dep::{DepState, Dependency, Resolver};
use libra_shared::git::GitRepo;

// path constants
static PATH_REPO: [&str; 2] = ["deps", "llvm-project"];

/// Get baseline cmake command
fn baseline_cmake_options() -> Vec<String> {
    vec![
        "-DCMAKE_BUILD_TYPE=Debug".into(),
        "-DBUILD_SHARED_LIBS=ON".into(),
        format!(
            "-DLLVM_ENABLE_PROJECTS={}",
            ["clang", "clang-tools-extra", "libc", "lld", "lldb", "polly",].join(";")
        ),
        format!(
            "-DLLVM_ENABLE_RUNTIMES={}",
            ["compiler-rt", "libcxx", "libcxxabi", "libunwind"].join(";")
        ),
        "-DLLVM_ENABLE_RTTI=ON".into(),
        "-DLLVM_ENABLE_LIBCXX=ON".into(),
        "-DLIBC_ENABLE_USE_BY_CLANG=ON".into(),
        "-DCLANG_DEFAULT_CXX_STDLIB=libc++".into(),
        #[cfg(target_os = "macos")]
        "-DCMAKE_OSX_ARCHITECTURES=arm64".into(),
        #[cfg(target_os = "macos")]
        "-DLLDB_USE_SYSTEM_DEBUGSERVER=ON".into(),
        #[cfg(target_os = "macos")]
        "-DLLVM_LIBC_MPFR_INSTALL_PATH=/opt/homebrew/Cellar/mpfr".into(),
    ]
}

/// Artifact path resolver for LLVM
pub struct ResolverLLVM {
    /// Base path for the artifact directory
    path_artifact: PathBuf,
    /// <artifact>/build
    path_build: PathBuf,
    /// <artifact>/build
    path_install: PathBuf,
}

impl Resolver for ResolverLLVM {
    fn construct(path: PathBuf) -> Self {
        Self {
            path_build: path.join("build"),
            path_install: path.join("install"),
            path_artifact: path,
        }
    }

    fn destruct(self) -> PathBuf {
        self.path_artifact
    }

    fn seek() -> Result<(GitRepo, Self)> {
        DepState::<ResolverLLVM, DepLLVM>::new()?.into_source_and_artifact()
    }
}

impl ResolverLLVM {
    pub fn path_build(&self) -> &Path {
        &self.path_build
    }

    pub fn path_install(&self) -> &Path {
        &self.path_install
    }
}

/// Represent the LLVM deps
pub struct DepLLVM {}

impl Dependency<ResolverLLVM> for DepLLVM {
    fn repo_path_from_root() -> &'static [&'static str] {
        &PATH_REPO
    }

    fn list_build_options(path_src: &Path, path_config: &Path) -> Result<()> {
        let mut cmd = Command::new("cmake");
        cmd.arg("-LAH")
            .args(baseline_cmake_options())
            .arg(path_src.join("llvm"))
            .current_dir(path_config);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }
        Ok(())
    }

    fn build(path_src: &Path, resolver: &ResolverLLVM) -> Result<()> {
        // config
        fs::create_dir(&resolver.path_build)?;
        let mut cmd = Command::new("cmake");
        cmd.arg("-G")
            .arg("Ninja")
            .args(baseline_cmake_options())
            .arg(path_src.join("llvm"))
            .current_dir(&resolver.path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Configure failed"));
        }

        // build
        let mut cmd = Command::new("cmake");
        cmd.arg("--build").arg(&resolver.path_build);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Build failed"));
        }

        // install
        fs::create_dir(&resolver.path_install)?;

        let mut cmd = Command::new("cmake");
        cmd.arg("--install")
            .arg(&resolver.path_build)
            .arg("--prefix")
            .arg(&resolver.path_install);
        let status = cmd.status()?;
        if !status.success() {
            return Err(anyhow!("Install failed"));
        }

        // done
        Ok(())
    }
}
