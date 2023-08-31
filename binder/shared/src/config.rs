use std::env;
use std::path::PathBuf;

use lazy_static::lazy_static;

#[cfg(target_os = "macos")]
use std::process::Command;

// paths
lazy_static! {
    pub static ref DOCERIZED: bool = matches!(env::var("DOCKER"), Ok(val) if val == "1");
    pub static ref PATH_ROOT: PathBuf = {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert!(path.pop());
        assert!(path.pop());
        path
    };
    pub static ref PATH_STUDIO: PathBuf = {
        let mut path = PATH_ROOT.join("studio");
        if *DOCERIZED {
            path.push("docker");
        } else {
            path.push("native");
        }
        path
    };
}

pub const TMPDIR_IN_STUDIO: &str = "tmp";

// platform-specific constants
#[cfg(target_os = "macos")]
lazy_static! {
    pub static ref UNAME_HARDWARE: String = {
        let cmd = Command::new("uname").arg("-m").output().expect("uname");
        if !cmd.status.success() || !cmd.stderr.is_empty() {
            panic!("uname");
        }
        String::from_utf8(cmd.stdout)
            .expect("uname")
            .trim()
            .to_string()
    };
    pub static ref UNAME_PLATFORM: String = {
        let cmd = Command::new("uname").arg("-s").output().expect("uname");
        if !cmd.status.success() || !cmd.stderr.is_empty() {
            panic!("uname");
        }
        String::from_utf8(cmd.stdout)
            .expect("uname")
            .trim()
            .to_string()
    };
    pub static ref UNAME_RELEASE: String = {
        let cmd = Command::new("uname").arg("-r").output().expect("uname");
        if !cmd.status.success() || !cmd.stderr.is_empty() {
            panic!("uname");
        }
        String::from_utf8(cmd.stdout)
            .expect("uname")
            .trim()
            .to_string()
    };
    pub static ref XCODE_SDK_PATH: String = {
        let cmd = Command::new("xcrun")
            .arg("--show-sdk-path")
            .output()
            .expect("xcode");
        if !cmd.status.success() || !cmd.stderr.is_empty() {
            panic!("xcode");
        }
        String::from_utf8(cmd.stdout)
            .expect("xcode")
            .trim()
            .to_string()
    };
}
