use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use walkdir::WalkDir;

use crate::proxy::{ClangArg, ClangInvocation, COMMAND_EXTENSION};

enum SysLib {
    C,
    Math,
    POSIXThread,
}

enum Language {
    C,
    CPP,
    Asm,
}

impl Language {
    pub fn probe(path: &Path) -> Option<Self> {
        let lang = match path.extension().and_then(|e| e.to_str())? {
            "c" => Self::C,
            "cpp" => Self::CPP,
            "cc" => Self::CPP,
            "s" => Self::Asm,
            _ => return None,
        };
        Some(lang)
    }
}

enum Action {
    Compile {
        input: PathBuf,
        lang: Language,
        output: PathBuf,
    },
    Link {
        inputs: Vec<PathBuf>,
        libs_sys: Vec<SysLib>,
        libs_usr: Vec<PathBuf>,
        output: PathBuf,
    },
    CompileAndLink {
        input: PathBuf,
        lang: Language,
        libs_sys: Vec<SysLib>,
        libs_usr: Vec<PathBuf>,
        output: PathBuf,
    },
}

impl Action {
    fn filter_args_for_output(invocation: ClangInvocation) -> Result<(ClangInvocation, PathBuf)> {
        let ClangInvocation { cwd, args } = invocation;

        let mut new_args = vec![];
        let mut target = None;
        for item in args {
            if let ClangArg::Output(name) = &item {
                if target.is_some() {
                    panic!("more than one output specified");
                }

                // resolve path
                let path = Path::new(name);
                let path_resolved = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    cwd.join(path)
                };
                if !path_resolved.exists() {
                    bail!("output path does not exist");
                }
                target = Some(path_resolved);
            } else {
                new_args.push(item);
            }
        }

        let output = match target {
            None => bail!("no output in the invocation"),
            Some(out) => out,
        };
        let new_invocation = ClangInvocation {
            cwd,
            args: new_args,
        };
        Ok((new_invocation, output))
    }

    fn filter_args_for_inputs(
        invocation: ClangInvocation,
    ) -> Result<(ClangInvocation, Vec<PathBuf>)> {
        let ClangInvocation { cwd, args } = invocation;

        let mut new_args = vec![];
        let mut inputs = vec![];
        for item in args {
            if let ClangArg::Input(name) = &item {
                // resolve path
                let path = Path::new(name);
                let path_resolved = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    cwd.join(path)
                };
                if !path_resolved.exists() {
                    bail!("input path does not exist");
                }
                inputs.push(path_resolved);
            } else {
                new_args.push(item);
            }
        }

        if inputs.is_empty() {
            bail!("no inputs in the invocation");
        }
        let new_invocation = ClangInvocation {
            cwd,
            args: new_args,
        };
        Ok((new_invocation, inputs))
    }

    fn filter_args_for_mode_compile(
        invocation: ClangInvocation,
    ) -> Result<(ClangInvocation, bool)> {
        let ClangInvocation { cwd, args } = invocation;

        let mut is_compile_only = false;
        let mut new_args = vec![];
        for item in args {
            if matches!(&item, ClangArg::ModeCompile) {
                if is_compile_only {
                    bail!("-c specified multiple times");
                }
                is_compile_only = true;
            } else {
                new_args.push(item);
            }
        }

        let new_invocation = ClangInvocation {
            cwd,
            args: new_args,
        };
        Ok((new_invocation, is_compile_only))
    }

    fn filter_args_for_mode_link(
        invocation: ClangInvocation,
    ) -> Result<(ClangInvocation, Option<(Vec<SysLib>, Vec<PathBuf>)>)> {
        let ClangInvocation { cwd, args } = invocation;

        // collect libraries
        let mut has_linking_flags = false;
        let mut lib_names = vec![];
        let mut lib_paths = vec![];
        let mut libs_sys = vec![];
        let mut new_args = vec![];
        for item in args {
            match &item {
                ClangArg::LibName(val) => {
                    has_linking_flags = true;

                    // resolve system libraries
                    match val.as_str() {
                        "c" => libs_sys.push(SysLib::C),
                        "m" => libs_sys.push(SysLib::Math),
                        "pthread" => libs_sys.push(SysLib::POSIXThread),
                        _ => lib_names.push(val.to_string()),
                    }
                }
                ClangArg::LibPath(val) => {
                    has_linking_flags = true;

                    // resolve path
                    let path = Path::new(val);
                    let path_resolved = if path.is_absolute() {
                        path.to_path_buf()
                    } else {
                        cwd.join(path)
                    };
                    if path_resolved.exists() {
                        lib_paths.push(path_resolved);
                    }
                }
                ClangArg::LinkStatic | ClangArg::LinkShared | ClangArg::Linker(..) => {
                    has_linking_flags = true;
                }
                _ => {
                    new_args.push(item);
                }
            }
        }

        // find requested libraries
        let libs = if has_linking_flags {
            let mut libs_usr = vec![];
            for name in lib_names {
                let mut found = false;
                for path in &lib_paths {
                    // TODO: resolve library
                    let lib_file = path.join(&name);
                    if lib_file.exists() {
                        if found {
                            bail!("more than one candidate for library {}", name);
                        }
                        found = true;
                        libs_usr.push(lib_file);
                    }
                }
                if !found {
                    bail!("library {} not found", name);
                }
            }

            Some((libs_sys, libs_usr))
        } else {
            None
        };

        let new_invocation = ClangInvocation {
            cwd,
            args: new_args,
        };
        Ok((new_invocation, libs))
    }

    fn parse(invocation: ClangInvocation) -> Result<Self> {
        let (invocation, output) = Self::filter_args_for_output(invocation)?;
        let (invocation, inputs) = Self::filter_args_for_inputs(invocation)?;
        let (invocation, is_compile_only) = Self::filter_args_for_mode_compile(invocation)?;
        let (invocation, link_libs_opt) = Self::filter_args_for_mode_link(invocation)?;

        // build action
        let action = if is_compile_only {
            if link_libs_opt.is_some() {
                bail!("unexpected linking flags in compile-only mode");
            }

            if inputs.len() != 1 {
                bail!("more than one inputs in compile-only mode ");
            }
            let input = inputs.into_iter().next().unwrap();
            let lang = match Language::probe(&input) {
                None => bail!("unrecognized source language"),
                Some(l) => l,
            };

            Action::Compile {
                input,
                lang,
                output,
            }
        } else if inputs.len() == 1 {
            let input = inputs.into_iter().next().unwrap();

            // at least linking is involved
            let (libs_sys, libs_usr) = link_libs_opt.unwrap_or_else(|| (vec![], vec![]));

            match Language::probe(&input) {
                None => {
                    // linking mode
                    Action::Link {
                        inputs: vec![input],
                        libs_sys,
                        libs_usr,
                        output,
                    }
                }
                Some(lang) => {
                    // compile and link mode
                    Action::CompileAndLink {
                        input,
                        lang,
                        libs_sys,
                        libs_usr,
                        output,
                    }
                }
            }
        } else {
            for item in &inputs {
                if Language::probe(item).is_some() {
                    bail!("found source code file in linking mode");
                }
            }

            let (libs_sys, libs_usr) = link_libs_opt.unwrap_or_else(|| (vec![], vec![]));

            Action::Link {
                inputs,
                libs_sys,
                libs_usr,
                output,
            }
        };

        Ok(action)
    }
}

/// Scan over the directory and collect build commands
pub fn analyze(path_src: &Path) -> Result<()> {
    // collect commands
    for entry in WalkDir::new(path_src) {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |e| e == COMMAND_EXTENSION) {
            let content = fs::read_to_string(path)?;
            let invocation: ClangInvocation = serde_json::from_str(&content)?;
            Action::parse(invocation)?;
        }
    }

    Ok(())
}
