use serde::{Deserialize, Serialize};

use crate::ir::adapter::constant::Constant;
use crate::ir::adapter::typing::Type;

#[derive(Serialize, Deserialize)]
pub enum Value {
    /// constant
    Constant(Constant),
    /// reference to an argument
    Argument { ty: Type, index: usize },
    /// reference to an instruction
    Instruction { ty: Type, index: usize },
}

#[derive(Serialize, Deserialize)]
pub struct InlineAsm {
    pub asm: String,
    pub constraint: String,
}