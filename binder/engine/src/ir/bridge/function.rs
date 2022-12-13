use crate::error::{EngineError, EngineResult, Unsupported};
use crate::ir::adapter;
use crate::ir::bridge::cfg::ControlFlowGraph;
use crate::ir::bridge::shared::{Identifier, SymbolRegistry};
use crate::ir::bridge::typing::{Type, TypeRegistry};

/// An adapted representation of an LLVM function
#[derive(Eq, PartialEq)]
pub struct Function {
    /// function name
    pub name: Identifier,
    /// parameter definitions
    pub params: Vec<(Option<Identifier>, Type)>,
    /// return type
    pub ret: Option<Type>,
    /// body of the function (in terms of a CFG)
    pub body: Option<ControlFlowGraph>,
}

impl Function {
    pub fn convert(
        func: &adapter::function::Function,
        typing: &TypeRegistry,
        symbols: &SymbolRegistry,
    ) -> EngineResult<Self> {
        let adapter::function::Function {
            name,
            ty,
            is_defined,
            is_exact,
            is_intrinsic,
            params,
            blocks,
        } = func;

        // filter out unsupported cases
        if !*is_exact {
            return Err(EngineError::NotSupportedYet(Unsupported::WeakFunction));
        }

        // convert the name
        let ident: Identifier = name
            .as_ref()
            .ok_or_else(|| EngineError::InvalidAssumption("no anonymous function".into()))?
            .into();

        // convert the signature
        let func_ty = typing.convert(ty)?;
        let (param_tys, ret_ty) = match func_ty {
            Type::Function { params, ret } => (params, ret.map(|e| *e)),
            _ => {
                return Err(EngineError::InvalidAssumption(format!(
                    "invalid signature for function: {}",
                    ident
                )));
            }
        };

        // convert parameters
        if params.len() != param_tys.len() {
            return Err(EngineError::InvalidAssumption(format!(
                "parameter count mismatch for function: {}",
                ident
            )));
        }
        let params_new: Vec<_> = params
            .iter()
            .zip(param_tys)
            .map(|(p, t)| {
                let adapter::function::Parameter {
                    name: param_name,
                    ty: param_ty,
                } = p;

                let param_ty_new = typing.convert(param_ty)?;
                if param_ty_new != t {
                    Err(EngineError::InvalidAssumption(format!(
                        "parameter type mismatch for function: {}",
                        ident
                    )))
                } else {
                    Ok((param_name.as_ref().map(|e| e.into()), t))
                }
            })
            .collect::<EngineResult<_>>()?;

        let body = if *is_defined {
            if blocks.is_empty() {
                return Err(EngineError::InvalidAssumption(format!(
                    "a defined function must have at least one basic block: {}",
                    ident
                )));
            }
            if *is_intrinsic {
                return Err(EngineError::InvalidAssumption(format!(
                    "a defined function cannot be an intrinsic: {}",
                    name.as_ref().map_or("<unknown>", |e| e.as_str())
                )));
            }

            // construct the CFG
            Some(ControlFlowGraph::build(
                typing,
                symbols,
                &params_new,
                ret_ty.as_ref(),
                blocks,
            )?)
        } else {
            None
        };

        // done with the construction
        Ok(Self {
            name: ident,
            params: params_new,
            ret: ret_ty,
            body,
        })
    }
}
