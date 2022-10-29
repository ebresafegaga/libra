use std::path::PathBuf;

pub use error::EngineError;

use crate::error::EngineResult;
use crate::flow::Workflow;

mod error;
mod flow;
mod ir;

/// Main entrypoint
pub fn analyze(inputs: Vec<PathBuf>, output: PathBuf) -> EngineResult<()> {
    let flow = Workflow::new(inputs, output);
    flow.execute()
}
