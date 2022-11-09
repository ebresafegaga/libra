use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use log::info;
use structopt::StructOpt;
use tempfile::tempdir;

use libra_engine::analyze;
use libra_shared::config::PATH_STUDIO;
use libra_shared::logging;

#[derive(StructOpt)]
#[structopt(
    name = "libra-engine",
    about = "The main execution engine for LIBRA",
    rename_all = "kebab-case"
)]
struct Args {
    /// Studio directory
    #[structopt(short, long)]
    studio: Option<PathBuf>,

    /// Verbosity
    #[structopt(short, long)]
    verbose: bool,

    /// Source code files
    #[structopt(required = true)]
    inputs: Vec<PathBuf>,

    /// Extra flags to be passed to clang
    #[structopt(short, long)]
    flags: Vec<String>,

    /// Limit the depth of fixedpoint optimization
    #[structopt(short, long)]
    depth: Option<usize>,

    /// Keep the workflow artifacts in the studio
    #[structopt(short, long)]
    keep: bool,
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let Args {
        studio,
        verbose,
        inputs,
        flags,
        depth,
        keep,
    } = args;
    let studio = studio.as_ref().unwrap_or(&PATH_STUDIO);

    // setup logging
    logging::setup(verbose)?;

    // decide on the workspace
    let (temp, output) = if keep {
        let path = studio.join("libra");
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        (None, path)
    } else {
        let dir = tempdir()?;
        let path = dir.path().to_path_buf();
        (Some(dir), path)
    };

    // run the analysis
    match analyze(depth, flags, inputs, output) {
        Ok(trace) => {
            info!("Number of optimization rounds: {}", trace.len());
        }
        Err(err) => {
            println!("{}", err);
        }
    };

    // drop temp dir explicitly
    match temp {
        None => (),
        Some(dir) => {
            dir.close()?;
        }
    };

    // done with everything
    Ok(())
}
