use std::path::PathBuf;
use std::process::Command;
use std::{env, fs, process};

use serde::{Deserialize, Serialize};

use libra_engine::flow::shared::Context;

/// Extension for our own command database
pub static COMMAND_EXTENSION: &str = ".command.json";

/// Extension for our own library mark
pub static LIBMARK_EXTENSION: &str = ".library.mark";

/// Clang arguments
#[derive(Serialize, Deserialize)]
pub enum ClangArg {
    /// -c
    ModeCompile,
    /// -std=<token>
    Standard(String),
    /// -D<key>{=<value>}
    Define(String, Option<String>),
    /// -I<token>, -I <token>
    Include(String),
    /// -isysroot <token>
    IncludeSysroot(String),
    /// -Wp,<arg>,<arg>,...
    Preprocessor(Vec<(String, Option<String>)>),
    /// -O<level>
    Optimization(String),
    /// -arch <token>
    Arch(String),
    /// -march=<token>
    MachineArch(String),
    /// -g, --debug
    Debug,
    /// -l<token>, -l <token>
    LibName(String),
    /// -L<token>, -L <token>
    LibPath(String),
    /// -shared, --shared
    LinkShared,
    /// -static, --static
    LinkStatic,
    /// -Wl,<arg>,<arg>,...
    Linker(Vec<(String, Option<String>)>),
    /// -mllvm <key>{=<value>}
    Backend(String, Option<String>),
    /// -fPIC
    FlagPIC,
    /// -fPIE
    FlagPIE,
    /// -W<key>{=<value>}
    Warning(String, Option<String>),
    /// -w, --no-warnings
    NoWarnings,
    /// -pedantic
    Pedantic,
    /// -pthread
    POSIXThread,
    /// -print-<key>{=<value>}, --print-<key>{=<value>}
    Print(String, Option<String>),
    /// -o <token>
    Output(String),
    /// <token>
    Input(String),
}

impl ClangArg {
    pub fn collect<'a, I>(mut iter: I) -> Vec<Self>
    where
        I: Iterator<Item = &'a str>,
    {
        let mut args = vec![];
        while let Some(token) = iter.next() {
            args.push(Self::parse(token, &mut iter));
        }
        args
    }

    fn parse<'a, I>(token: &'a str, stream: &mut I) -> Self
    where
        I: Iterator<Item = &'a str>,
    {
        if !token.starts_with('-') {
            return Self::Input(token.to_string());
        }

        match token {
            "-c" => {
                return Self::ModeCompile;
            }
            "-I" => {
                return Self::Include(Self::expect_next(stream));
            }
            "-isysroot" => {
                return Self::IncludeSysroot(Self::expect_next(stream));
            }
            "-l" => {
                return Self::LibName(Self::expect_next(stream));
            }
            "-L" => {
                return Self::LibPath(Self::expect_next(stream));
            }
            "-arch" => {
                return Self::Arch(Self::expect_next(stream));
            }
            "-g" | "--debug" => {
                return Self::Debug;
            }
            "-shared" | "--shared" => {
                return Self::LinkShared;
            }
            "-static" | "--static" => {
                return Self::LinkStatic;
            }
            "-mllvm" => {
                let (k, v) = Self::expect_maybe_key_value(&Self::expect_next(stream));
                return Self::Backend(k, v);
            }
            "-fPIC" => {
                return Self::FlagPIC;
            }
            "-fPIE" => {
                return Self::FlagPIE;
            }
            "-w" | "--no-warnings" => {
                return Self::NoWarnings;
            }
            "-pedantic" => {
                return Self::Pedantic;
            }
            "-pthread" => {
                return Self::POSIXThread;
            }
            "-o" => {
                return Self::Output(Self::expect_next(stream));
            }
            _ => (),
        }

        if let Some(inner) = token.strip_prefix("-std=") {
            return Self::Standard(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-D") {
            let (k, v) = Self::expect_maybe_key_value(inner);
            return Self::Define(k, v);
        }
        if let Some(inner) = token.strip_prefix("-I") {
            return Self::Include(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-Wp,") {
            let mut args = vec![];
            for item in inner.split(',') {
                let (k, v) = Self::expect_maybe_key_value(item);
                args.push((k, v));
            }
            return Self::Preprocessor(args);
        }
        if let Some(inner) = token.strip_prefix("-O") {
            return Self::Optimization(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-march=") {
            return Self::MachineArch(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-l") {
            return Self::LibName(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-L") {
            return Self::LibPath(inner.to_string());
        }
        if let Some(inner) = token.strip_prefix("-Wl,") {
            let mut args = vec![];
            for item in inner.split(',') {
                let (k, v) = Self::expect_maybe_key_value(item);
                args.push((k, v));
            }
            return Self::Linker(args);
        }
        if let Some(inner) = token.strip_prefix("-W") {
            let (k, v) = Self::expect_maybe_key_value(inner);
            return Self::Warning(k, v);
        }
        if let Some(inner) = token
            .strip_prefix("-print-")
            .or_else(|| token.strip_prefix("--print-"))
        {
            let (k, v) = Self::expect_maybe_key_value(inner);
            return Self::Print(k, v);
        }

        panic!("unknown Clang option: {}", token);
    }

    fn expect_next<'a, I>(stream: &mut I) -> String
    where
        I: Iterator<Item = &'a str>,
    {
        stream.next().expect("token").to_string()
    }

    fn expect_maybe_key_value(item: &str) -> (String, Option<String>) {
        match item.find('=') {
            None => (item.to_string(), None),
            Some(index) => {
                let (key, val) = item.split_at(index);
                let val = val.strip_prefix('=').unwrap();
                (key.to_string(), Some(val.to_string()))
            }
        }
    }
}

impl ClangArg {
    fn as_args(&self) -> Vec<String> {
        match self {
            Self::ModeCompile => vec!["-c".into()],
            Self::Standard(val) => vec![format!("-std={}", val)],
            Self::Define(key, None) => vec![format!("-D{}", key)],
            Self::Define(key, Some(val)) => vec![format!("-D{}={}", key, val)],
            Self::Include(val) => vec![format!("-I{}", val)],
            Self::IncludeSysroot(val) => vec!["-isysroot".into(), val.into()],
            Self::Preprocessor(args) => {
                vec![format!(
                    "-Wp,{}",
                    args.iter()
                        .map(|(k, v)| {
                            match v {
                                None => k.to_string(),
                                Some(v) => format!("{}={}", k, v),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                )]
            }
            Self::Optimization(val) => vec![format!("-O{}", val)],
            Self::Arch(val) => vec!["-arch".into(), val.into()],
            Self::MachineArch(val) => vec![format!("-march={}", val)],
            Self::Debug => vec!["-g".into()],
            Self::LibName(val) => vec![format!("-l{}", val)],
            Self::LibPath(val) => vec![format!("-L{}", val)],
            Self::LinkShared => vec!["-shared".into()],
            Self::LinkStatic => vec!["-static".into()],
            Self::Linker(args) => {
                vec![format!(
                    "-Wl,{}",
                    args.iter()
                        .map(|(k, v)| {
                            match v {
                                None => k.to_string(),
                                Some(v) => format!("{}={}", k, v),
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                )]
            }
            Self::Backend(key, None) => vec!["-mllvm".into(), key.into()],
            Self::Backend(key, Some(val)) => vec!["-mllvm".into(), format!("{}={}", key, val)],
            Self::FlagPIC => vec!["-fPIC".into()],
            Self::FlagPIE => vec!["-fPIE".into()],
            Self::Warning(key, None) => vec![format!("-W{}", key)],
            Self::Warning(key, Some(val)) => vec![format!("-W{}={}", key, val)],
            Self::NoWarnings => vec!["-w".into()],
            Self::Pedantic => vec!["-pedantic".into()],
            Self::POSIXThread => vec!["-pthread".into()],
            Self::Print(key, None) => vec![format!("-print-{}", key)],
            Self::Print(key, Some(val)) => vec![format!("-print-{}={}", key, val)],
            Self::Output(val) => vec![format!("-o {}", val)],
            Self::Input(val) => vec![format!("unexpected input {}", val)],
        }
    }
}

/// Clang invocation
#[derive(Serialize, Deserialize)]
pub struct ClangInvocation {
    pub cwd: PathBuf,
    pub cxx: bool,
    pub args: Vec<ClangArg>,
}

/// Wrap a clang tool
pub fn proxy_clang(cxx: bool) {
    // get paths
    let ctxt = Context::new().expect("LLVM context");
    let name = if cxx { "clang++" } else { "clang" };
    let bin_clang = ctxt.path_llvm(["bin", name]).expect("ascii path only");

    // collect arguments
    let args: Vec<_> = env::args().skip(1).collect();

    // pass-through the arguments and execute the command first
    let status = Command::new(bin_clang)
        .args(&args)
        .status()
        .expect("command execution");
    if !status.success() {
        process::exit(status.code().expect("status code"))
    }

    // only process arguments upon successful invocation
    let parsed = ClangArg::collect(args.iter().map(|s| s.as_str()));

    // check output
    let mut output = None;
    for item in &parsed {
        if let ClangArg::Output(out) = item {
            if output.is_some() {
                panic!("more than one output specified: {}", args.join(" "));
            }
            output = Some(out);
        }
    }
    let path = match output {
        None => return,
        Some(out) => format!("{}{}", out, COMMAND_EXTENSION),
    };

    // create the invocation package
    let invocation = ClangInvocation {
        cwd: env::current_dir()
            .expect("unable to get current working directory")
            .canonicalize()
            .expect("unable to get canonicalize cwd path"),
        cxx,
        args: parsed,
    };

    // serialize
    let content = serde_json::to_string_pretty(&invocation).expect("serialization error");
    fs::write(path, content).expect("IO error");
}
