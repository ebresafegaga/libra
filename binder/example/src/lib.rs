mod apps;

use anyhow::Result;
use structopt::StructOpt;

use libra_shared::config::initialize;

#[derive(StructOpt)]
enum Example {
    ApacheHttpd,
}

#[derive(StructOpt)]
#[structopt(
    name = "libra-example",
    about = "A driver for LIBRA workflow on example projects",
    rename_all = "kebab-case"
)]
struct Args {
    /// Example
    #[structopt(subcommand)]
    example: Example,
}

/// Main entrypoint
pub fn entrypoint() -> Result<()> {
    // setup
    let args = Args::from_args();
    let Args { example } = args;
    initialize();

    // run the subcommand
    match example {
        Example::ApacheHttpd => (),
    }

    Ok(())
}
