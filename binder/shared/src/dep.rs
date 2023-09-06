use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use anyhow::Result;
use log::{info, warn};
use tempfile::tempdir;

use crate::config::PATH_ROOT;
use crate::git::GitRepo;

/// A trait that marks a dependency in the project
pub trait Dependency {
    /// Location of the git repo from the project root
    fn repo_path_from_root() -> &'static [&'static str];

    /// List configurable options for building
    fn list_build_options(path_src: &Path, path_build: &Path) -> Result<()>;

    /// Build the deps from scratch, install to artifact directory if needed
    fn build(path_src: &Path, path_build: &Path, path_install: Option<&Path>) -> Result<()>;
}

/// A struct that represents the build-from-scratch state
pub struct Scratch<T: Dependency> {
    repo: GitRepo,
    artifact: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T: Dependency> Scratch<T> {
    /// Build the deps from scratch
    pub fn make(self, workdir: Option<&Path>) -> Result<Package<T>> {
        let Self {
            repo,
            artifact,
            _phantom,
        } = self;

        // build
        match workdir {
            None => T::build(repo.path(), &artifact, None)?,
            Some(path) => T::build(repo.path(), path, Some(&artifact))?,
        }

        // done with the building procedure
        Ok(Package {
            repo,
            artifact,
            _phantom,
        })
    }
}

/// A struct that represents the package-ready state
pub struct Package<T: Dependency> {
    repo: GitRepo,
    artifact: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T: Dependency> Package<T> {
    /// Destroy the deps so that we can build it again
    pub fn destroy(self) -> Result<Scratch<T>> {
        let Self {
            repo,
            artifact,
            _phantom,
        } = self;
        fs::remove_dir_all(&artifact)?;
        Ok(Scratch {
            repo,
            artifact,
            _phantom,
        })
    }

    /// Get the git repo from the package
    pub fn git_repo(&self) -> &GitRepo {
        &self.repo
    }

    /// Get the artifact path from the package
    pub fn artifact_path(&self) -> &Path {
        &self.artifact
    }
}

/// Automatically differentiate the scratch and package version of LLVM
pub enum DepState<T: Dependency> {
    Scratch(Scratch<T>),
    Package(Package<T>),
}

impl<T: Dependency> DepState<T> {
    /// Get the deps state
    pub fn new(studio: &Path, version: Option<&str>) -> Result<Self> {
        // derive the correct path
        let segments = T::repo_path_from_root();

        let mut repo_path = PATH_ROOT.clone();
        repo_path.extend(segments);
        let repo = GitRepo::new(repo_path, version)?;

        let mut artifact = studio.to_path_buf();
        artifact.extend(segments);
        artifact.push(repo.commit());

        // check the existence of the pre-built package
        let state = if artifact.exists() {
            Self::Package(Package {
                repo,
                artifact,
                _phantom: PhantomData,
            })
        } else {
            Self::Scratch(Scratch {
                repo,
                artifact,
                _phantom: PhantomData,
            })
        };

        // done
        Ok(state)
    }

    /// List the possible build options
    pub fn list_build_options(self) -> Result<()> {
        let repo = match self {
            Self::Scratch(Scratch { repo, .. }) => repo,
            Self::Package(Package { repo, .. }) => repo,
        };

        // always happens in tmpfs
        let tmp = tempdir()?;
        T::list_build_options(repo.path(), tmp.path())?;
        tmp.close()?;

        Ok(())
    }

    /// Build the package
    pub fn build(self, workdir: Option<&Path>, force: bool) -> Result<()> {
        let scratch = match self {
            DepState::Scratch(scratch) => scratch,
            DepState::Package(package) => {
                if !force {
                    info!("Package already exists");
                    return Ok(());
                } else {
                    warn!("Force rebuilding package");
                    package.destroy()?
                }
            }
        };
        scratch.make(workdir)?;
        Ok(())
    }
}
