use num_traits::ToPrimitive;

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{EngineError, EngineResult, Unsupported};
use crate::ir::adapter;
use crate::ir::bridge::constant::Constant;
use crate::ir::bridge::shared::{Identifier, SymbolRegistry};
use crate::ir::bridge::typing::{Type, TypeRegistry};
use crate::ir::bridge::value::{BlockLabel, RegisterSlot, Value};

/// An naive translation of an LLVM instruction
#[derive(Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Instruction {
    // memory access
    Alloca {
        base_type: Type,
        size: Option<Value>,
        result: RegisterSlot,
    },
    Load {
        pointee_type: Type,
        pointer: Value,
        result: RegisterSlot,
    },
    Store {
        pointee_type: Type,
        pointer: Value,
        value: Value,
    },
    // call
    CallDirect {
        function: Identifier,
        args: Vec<Value>,
        result: Option<(Type, RegisterSlot)>,
    },
    CallIndirect {
        callee: Value,
        args: Vec<Value>,
        result: Option<(Type, RegisterSlot)>,
    },
    // binary
    Binary {
        bits: usize,
        opcode: BinaryOperator,
        lhs: Value,
        rhs: Value,
        result: RegisterSlot,
    },
    // compare
    Compare {
        bits: Option<usize>, // some for bitvec and none for pointer
        predicate: ComparePredicate,
        lhs: Value,
        rhs: Value,
        result: RegisterSlot,
    },
    // cast
    CastBitvec {
        bits_from: usize,
        bits_into: usize,
        operand: Value,
        result: RegisterSlot,
    },
    CastPtrToBitvec {
        bits_into: usize,
        operand: Value,
        result: RegisterSlot,
    },
    CastBitvecToPtr {
        bits_from: usize,
        operand: Value,
        result: RegisterSlot,
    },
    CastPtr {
        operand: Value,
        result: RegisterSlot,
    },
    // freeze
    FreezePtr,
    FreezeBitvec {
        bits: usize,
    },
    FreezeNop {
        value: Value,
    },
    // GEP
    GEP {
        src_pointee_type: Type,
        dst_pointee_type: Type,
        pointer: Value,
        offset: Value,
        indices: Vec<Value>,
        result: RegisterSlot,
    },
    // selection
    ITE {
        cond: Value,
        then_value: Value,
        else_value: Value,
        result: RegisterSlot,
    },
    Phi {
        ty: Type,
        options: BTreeMap<BlockLabel, Value>,
        result: RegisterSlot,
    },
    // aggregation
    GetValue {
        src_ty: Type,
        dst_ty: Type,
        aggregate: Value,
        indices: Vec<usize>,
        result: RegisterSlot,
    },
    SetValue {
        src_ty: Type,
        dst_ty: Type,
        aggregate: Value,
        value: Value,
        indices: Vec<usize>,
        result: RegisterSlot,
    },
}

#[derive(Eq, PartialEq, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

impl BinaryOperator {
    pub fn parse(opcode: &str) -> EngineResult<Self> {
        let parsed = match opcode {
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "udiv" | "sdiv" => Self::Div,
            "urem" | "srem" => Self::Mod,
            "shl" => Self::Shl,
            "lshr" | "ashr" => Self::Shr,
            "and" => Self::And,
            "or" => Self::Or,
            "xor" => Self::Xor,
            "fadd" | "fsub" | "fmul" | "fdiv" | "frem" => {
                return Err(EngineError::NotSupportedYet(Unsupported::FloatingPoint))
            }
            _ => {
                return Err(EngineError::InvalidAssumption(format!(
                    "unexpected binary opcode: {}",
                    opcode
                )));
            }
        };
        Ok(parsed)
    }
}

#[derive(Eq, PartialEq, Clone)]
pub enum ComparePredicate {
    EQ,
    NE,
    GT,
    GE,
    LT,
    LE,
}

impl ComparePredicate {
    pub fn parse(opcode: &str) -> EngineResult<Self> {
        let parsed = match opcode {
            "i_eq" => Self::EQ,
            "i_ne" => Self::NE,
            "i_ugt" | "i_sgt" => Self::GT,
            "i_uge" | "i_sge" => Self::GE,
            "i_ult" | "i_slt" => Self::LT,
            "i_ule" | "i_sle" => Self::LE,
            "f_f" | "f_oeq" | "f_ogt" | "f_oge" | "f_olt" | "f_ole" | "f_one" | "f_ord"
            | "f_uno" | "f_ueq" | "f_ugt" | "f_uge" | "f_ult" | "f_ule" | "f_une" | "f_t" => {
                return Err(EngineError::NotSupportedYet(Unsupported::FloatingPoint))
            }
            _ => {
                return Err(EngineError::InvalidAssumption(format!(
                    "unexpected compare predicate: {}",
                    opcode
                )));
            }
        };
        Ok(parsed)
    }
}

/// An naive translation of an LLVM terminator instruction
#[derive(Eq, PartialEq)]
pub enum Terminator {
    /// function return
    Return { val: Option<Value> },
    /// unconditional branch
    Goto { target: BlockLabel },
    /// conditional branch
    Branch {
        cond: Value,
        then_case: BlockLabel,
        else_case: BlockLabel,
    },
    /// switch
    Switch {
        cond: Value,
        cases: BTreeMap<u64, BlockLabel>,
        default: Option<BlockLabel>,
    },
    /// enters an unreachable state
    Unreachable,
}

/// A context manager for converting instructions
pub struct Context<'a> {
    pub typing: &'a TypeRegistry,
    pub symbols: &'a SymbolRegistry,
    pub blocks: BTreeSet<usize>,
    pub insts: BTreeMap<usize, Option<Type>>,
    pub args: BTreeMap<usize, Type>,
    pub ret: Option<Type>,
}

impl<'a> Context<'a> {
    /// convert a value
    pub fn parse_value(
        &mut self,
        val: &adapter::value::Value,
        expected_type: &Type,
    ) -> EngineResult<Value> {
        use adapter::value::Value as AdaptedValue;

        let converted = match val {
            AdaptedValue::Constant(constant) => Value::Constant(Constant::convert(
                constant,
                expected_type,
                self.typing,
                self.symbols,
            )?),
            AdaptedValue::Argument { ty, index } => {
                let actual_ty = self.typing.convert(ty)?;
                if expected_type != &actual_ty {
                    return Err(EngineError::InvariantViolation(
                        "argument type mismatch".into(),
                    ));
                }
                match self.args.get(index) {
                    None => {
                        return Err(EngineError::InvariantViolation(
                            "invalid argument index".into(),
                        ));
                    }
                    Some(arg_type) => {
                        if arg_type != &actual_ty {
                            return Err(EngineError::InvariantViolation(
                                "param type mismatch".into(),
                            ));
                        }
                    }
                }
                Value::Argument {
                    index: index.into(),
                    ty: actual_ty,
                }
            }
            AdaptedValue::Instruction { ty, index } => {
                let actual_ty = self.typing.convert(ty)?;
                if expected_type != &actual_ty {
                    return Err(EngineError::InvariantViolation(
                        "instruction type mismatch".into(),
                    ));
                }
                match self.insts.insert(*index, Some(actual_ty.clone())) {
                    None => {
                        return Err(EngineError::InvariantViolation(
                            "invalid instruction index".into(),
                        ));
                    }
                    Some(None) => {
                        // first time registration
                    }
                    Some(Some(reg_type)) => {
                        // check type consistency
                        if reg_type != actual_ty {
                            return Err(EngineError::InvariantViolation(
                                "register type mismatch".into(),
                            ));
                        }
                    }
                }
                Value::Register {
                    index: index.into(),
                    ty: actual_ty,
                }
            }
        };
        Ok(converted)
    }

    /// convert a value in either bv32 or bv64
    pub fn parse_value_bv32_or_bv64(&mut self, val: &adapter::value::Value) -> EngineResult<Value> {
        match val.get_type() {
            adapter::typing::Type::Int { width: 32 } => {
                self.parse_value(val, &Type::Bitvec { bits: 32 })
            }
            _ => self.parse_value(val, &Type::Bitvec { bits: 64 }),
        }
    }

    /// convert an instruction
    pub fn parse_instruction(
        &mut self,
        inst: &adapter::instruction::Instruction,
    ) -> EngineResult<Instruction> {
        use adapter::instruction::Inst as AdaptedInst;
        use adapter::typing::Type as AdaptedType;

        let adapter::instruction::Instruction {
            name: _,
            ty,
            index,
            repr,
        } = inst;

        let item = match repr {
            // memory access
            AdaptedInst::Alloca {
                allocated_type,
                size,
                address_space,
            } => {
                let inst_ty = self.typing.convert(ty)?;
                if !matches!(inst_ty, Type::Pointer) {
                    return Err(EngineError::InvalidAssumption(
                        "AllocaInst should return a pointer type".into(),
                    ));
                }
                if *address_space != 0 {
                    return Err(EngineError::NotSupportedYet(
                        Unsupported::PointerAddressSpace,
                    ));
                }
                let base_type = self.typing.convert(allocated_type)?;
                let size_new = match size.as_ref() {
                    None => None,
                    Some(val) => Some(self.parse_value(val, &Type::Bitvec { bits: 64 })?),
                };
                Instruction::Alloca {
                    base_type,
                    size: size_new,
                    result: index.into(),
                }
            }
            AdaptedInst::Load {
                pointee_type,
                pointer,
                ordering,
                address_space,
            } => {
                if ordering != "not_atomic" {
                    return Err(EngineError::NotSupportedYet(Unsupported::AtomicInstruction));
                }
                if *address_space != 0 {
                    return Err(EngineError::NotSupportedYet(
                        Unsupported::PointerAddressSpace,
                    ));
                }

                let inst_ty = self.typing.convert(ty)?;
                let pointee_type_new = self.typing.convert(pointee_type)?;
                if inst_ty != pointee_type_new {
                    return Err(EngineError::InvalidAssumption(
                        "LoadInst mismatch between result type and pointee type".into(),
                    ));
                }
                let pointer_new = self.parse_value(pointer, &Type::Pointer)?;
                Instruction::Load {
                    pointee_type: pointee_type_new,
                    pointer: pointer_new,
                    result: index.into(),
                }
            }
            AdaptedInst::Store {
                pointee_type,
                pointer,
                value,
                ordering,
                address_space,
            } => {
                if ordering != "not_atomic" {
                    return Err(EngineError::NotSupportedYet(Unsupported::AtomicInstruction));
                }
                if *address_space != 0 {
                    return Err(EngineError::NotSupportedYet(
                        Unsupported::PointerAddressSpace,
                    ));
                }
                if !matches!(ty, AdaptedType::Void) {
                    return Err(EngineError::InvalidAssumption(
                        "StoreInst should have void type".into(),
                    ));
                }

                let pointee_type_new = self.typing.convert(pointee_type)?;
                let pointer_new = self.parse_value(pointer, &Type::Pointer)?;
                let value_new = self.parse_value(value, &pointee_type_new)?;
                Instruction::Store {
                    pointee_type: pointee_type_new,
                    pointer: pointer_new,
                    value: value_new,
                }
            }
            AdaptedInst::VAArg { .. } => {
                return Err(EngineError::NotSupportedYet(Unsupported::VariadicArguments));
            }
            // calls
            AdaptedInst::CallDirect {
                callee,
                target_type,
                args,
            }
            | AdaptedInst::CallIndirect {
                callee,
                target_type,
                args,
            }
            | AdaptedInst::Intrinsic {
                callee,
                target_type,
                args,
            } => {
                let func_ty = self.typing.convert(target_type)?;
                match &func_ty {
                    Type::Function { params, ret } => {
                        if params.len() != args.len() {
                            return Err(EngineError::InvalidAssumption(
                                "CallInst number of arguments mismatch".into(),
                            ));
                        }
                        let args_new: Vec<_> = params
                            .iter()
                            .zip(args.iter())
                            .map(|(t, v)| self.parse_value(v, t))
                            .collect::<EngineResult<_>>()?;
                        let ret_ty = match ret {
                            None => {
                                if !matches!(ty, AdaptedType::Void) {
                                    return Err(EngineError::InvalidAssumption(
                                        "CallInst return type mismatch".into(),
                                    ));
                                }
                                None
                            }
                            Some(t) => {
                                let inst_ty = self.typing.convert(ty)?;
                                if t.as_ref() != &inst_ty {
                                    return Err(EngineError::InvalidAssumption(
                                        "CallInst return type mismatch".into(),
                                    ));
                                }
                                Some(inst_ty)
                            }
                        };
                        let callee_new = self.parse_value(callee, &Type::Pointer)?;
                        // TODO: better distinguish calls
                        if matches!(
                            repr,
                            AdaptedInst::CallDirect { .. } | AdaptedInst::Intrinsic { .. }
                        ) {
                            match callee_new {
                                Value::Constant(Constant::Function { name: callee_name }) => {
                                    Instruction::CallDirect {
                                        function: callee_name,
                                        args: args_new,
                                        result: ret_ty.map(|t| (t, index.into())),
                                    }
                                }
                                _ => {
                                    return Err(EngineError::InvalidAssumption(
                                        "direct or intrinsic call should target a named function"
                                            .into(),
                                    ));
                                }
                            }
                        } else {
                            if !matches!(repr, AdaptedInst::CallIndirect { .. }) {
                                return Err(EngineError::InvariantViolation(
                                    "expecting an indirect call but found some other call type"
                                        .into(),
                                ));
                            }
                            if matches!(callee_new, Value::Constant(Constant::Function { .. })) {
                                return Err(EngineError::InvalidAssumption(
                                    "indirect call should not target a named function".into(),
                                ));
                            }
                            Instruction::CallIndirect {
                                callee: callee_new,
                                args: args_new,
                                result: ret_ty.map(|t| (t, index.into())),
                            }
                        }
                    }
                    _ => {
                        return Err(EngineError::InvalidAssumption(
                            "CallInst refer to a non-function callee".into(),
                        ));
                    }
                }
            }
            AdaptedInst::Asm { .. } => {
                return Err(EngineError::NotSupportedYet(Unsupported::InlineAssembly));
            }
            // unary
            AdaptedInst::Unary { opcode, operand: _ } => match opcode.as_str() {
                "fneg" => {
                    return Err(EngineError::NotSupportedYet(Unsupported::FloatingPoint));
                }
                _ => {
                    return Err(EngineError::InvalidAssumption(format!(
                        "unexpected unary opcode: {}",
                        opcode
                    )));
                }
            },
            // binary
            AdaptedInst::Binary { opcode, lhs, rhs } => {
                let inst_ty = self.typing.convert(ty)?;
                let bits = match &inst_ty {
                    Type::Bitvec { bits } => *bits,
                    _ => {
                        return Err(EngineError::InvalidAssumption(
                            "binary operator has non-bitvec instruction type".into(),
                        ));
                    }
                };
                let opcode_parsed = BinaryOperator::parse(opcode)?;
                let lhs_new = self.parse_value(lhs, &inst_ty)?;
                let rhs_new = self.parse_value(rhs, &inst_ty)?;
                Instruction::Binary {
                    bits,
                    opcode: opcode_parsed,
                    lhs: lhs_new,
                    rhs: rhs_new,
                    result: index.into(),
                }
            }
            // comparison
            AdaptedInst::Compare {
                predicate,
                operand_type,
                lhs,
                rhs,
            } => {
                let inst_ty = self.typing.convert(ty)?;
                match &inst_ty {
                    Type::Bitvec { bits } => {
                        if *bits != 1 {
                            return Err(EngineError::InvalidAssumption(
                                "compare inst has non-bool instruction type".into(),
                            ));
                        }
                    }
                    _ => {
                        return Err(EngineError::InvalidAssumption(
                            "compare inst has non-bitvec instruction type".into(),
                        ));
                    }
                };
                let operand_ty = self.typing.convert(operand_type)?;
                let bits = match &operand_ty {
                    Type::Bitvec { bits } => Some(*bits),
                    Type::Pointer => None,
                    _ => {
                        return Err(EngineError::InvalidAssumption(
                            "compare inst has operand type that is neither bitvec or ptr".into(),
                        ));
                    }
                };
                let predicate_parsed = ComparePredicate::parse(predicate)?;
                let lhs_new = self.parse_value(lhs, &operand_ty)?;
                let rhs_new = self.parse_value(rhs, &operand_ty)?;
                Instruction::Compare {
                    bits,
                    predicate: predicate_parsed,
                    lhs: lhs_new,
                    rhs: rhs_new,
                    result: index.into(),
                }
            }
            // casts
            AdaptedInst::Cast {
                opcode,
                src_ty,
                dst_ty,
                src_address_space,
                dst_address_space,
                operand,
            } => {
                let inst_ty = self.typing.convert(ty)?;
                let src_ty_new = self.typing.convert(src_ty)?;
                let dst_ty_new = self.typing.convert(dst_ty)?;
                if dst_ty_new != inst_ty {
                    return Err(EngineError::InvariantViolation(
                        "type mismatch between dst type and inst type for cast".into(),
                    ));
                }
                let operand_new = self.parse_value(operand, &src_ty_new)?;
                match opcode.as_str() {
                    "trunc" | "zext" | "sext" => match (src_ty_new, dst_ty_new) {
                        (Type::Bitvec { bits: bits_from }, Type::Bitvec { bits: bits_into }) => {
                            Instruction::CastBitvec {
                                bits_from,
                                bits_into,
                                operand: operand_new,
                                result: index.into(),
                            }
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "expect bitvec type for bitvec cast".into(),
                            ));
                        }
                    },
                    "ptr_to_int" => match (src_ty_new, dst_ty_new) {
                        (Type::Pointer, Type::Bitvec { bits: bits_into }) => {
                            match src_address_space {
                                None => {
                                    return Err(EngineError::InvalidAssumption(
                                        "expect (src address_space) for ptr_to_int cast".into(),
                                    ));
                                }
                                Some(0) => Instruction::CastPtrToBitvec {
                                    bits_into,
                                    operand: operand_new,
                                    result: index.into(),
                                },
                                Some(_) => {
                                    return Err(EngineError::NotSupportedYet(
                                        Unsupported::PointerAddressSpace,
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "expect (ptr, bitvec) for ptr_to_int cast".into(),
                            ));
                        }
                    },
                    "int_to_ptr" => match (src_ty_new, dst_ty_new) {
                        (Type::Bitvec { bits: bits_from }, Type::Pointer) => {
                            match dst_address_space {
                                None => {
                                    return Err(EngineError::InvalidAssumption(
                                        "expect (dst address_space) for int_to_ptr cast".into(),
                                    ));
                                }
                                Some(0) => Instruction::CastBitvecToPtr {
                                    bits_from,
                                    operand: operand_new,
                                    result: index.into(),
                                },
                                Some(_) => {
                                    return Err(EngineError::NotSupportedYet(
                                        Unsupported::PointerAddressSpace,
                                    ));
                                }
                            }
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "expect (bitvec, ptr) for int_to_ptr cast".into(),
                            ));
                        }
                    },
                    "bitcast" => match (src_ty_new, dst_ty_new) {
                        (Type::Pointer, Type::Pointer) => Instruction::CastPtr {
                            operand: operand_new,
                            result: index.into(),
                        },
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "expect ptr type for bitcast".into(),
                            ));
                        }
                    },
                    "address_space_cast" => {
                        return Err(EngineError::NotSupportedYet(
                            Unsupported::PointerAddressSpace,
                        ));
                    }
                    _ => {
                        return Err(EngineError::InvalidAssumption(format!(
                            "unexpected cast opcode: {}",
                            opcode
                        )));
                    }
                }
            }
            // freeze
            AdaptedInst::Freeze { operand } => {
                let inst_ty = self.typing.convert(ty)?;
                let operand_new = self.parse_value(operand, &inst_ty)?;
                match operand_new {
                    Value::Constant(Constant::UndefBitvec { bits }) => {
                        Instruction::FreezeBitvec { bits }
                    }
                    Value::Constant(Constant::UndefPointer) => Instruction::FreezePtr,
                    // TODO(mengxu): freeze instruction should only be possible on undef,
                    // and yet, we still see freeze being applied to instruction values, e.g.,
                    // - %1 = load i32, ptr @loop_2
                    // - %.fr = freeze i32 %1
                    // - %cmp13 = icmp sgt i32 %.fr, 0
                    // Marking these cases as no-op here.
                    v => Instruction::FreezeNop { value: v },
                }
            }
            // GEP
            AdaptedInst::GEP {
                src_pointee_ty,
                dst_pointee_ty,
                pointer,
                indices,
                address_space,
            } => {
                if *address_space != 0 {
                    return Err(EngineError::NotSupportedYet(
                        Unsupported::PointerAddressSpace,
                    ));
                }

                let inst_ty = self.typing.convert(ty)?;
                if !matches!(inst_ty, Type::Pointer) {
                    return Err(EngineError::InvalidAssumption(
                        "GEP should return a pointer type".into(),
                    ));
                }

                let src_ty = self.typing.convert(src_pointee_ty)?;
                let dst_ty = self.typing.convert(dst_pointee_ty)?;

                // walk-down the tree
                if indices.is_empty() {
                    return Err(EngineError::InvalidAssumption(
                        "GEP contains no index".into(),
                    ));
                }

                let offset = indices.first().unwrap();
                let offset_new = self.parse_value_bv32_or_bv64(offset)?;

                let mut cur_ty = &src_ty;
                let mut indices_new = vec![];
                for idx in indices.iter().skip(1) {
                    let next_cur_ty = match cur_ty {
                        Type::Struct { name: _, fields } => {
                            let idx_new = self.parse_value(idx, &Type::Bitvec { bits: 32 })?;
                            let field_offset = match &idx_new {
                                Value::Constant(Constant::Bitvec {
                                    bits: _,
                                    value: field_offset,
                                }) => match field_offset.to_usize() {
                                    None => {
                                        return Err(EngineError::InvariantViolation(
                                            "field number must be within the range of usize".into(),
                                        ));
                                    }
                                    Some(v) => v,
                                },
                                _ => {
                                    return Err(EngineError::InvalidAssumption(
                                        "field number must be bv32".into(),
                                    ));
                                }
                            };
                            if field_offset >= fields.len() {
                                return Err(EngineError::InvalidAssumption(
                                    "field number out of range".into(),
                                ));
                            }
                            indices_new.push(idx_new);
                            fields.get(field_offset).unwrap()
                        }
                        Type::Array { element, length: _ } => {
                            let idx_new = self.parse_value_bv32_or_bv64(idx)?;
                            indices_new.push(idx_new);
                            element.as_ref()
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "GEP only applies to array and struct".into(),
                            ));
                        }
                    };
                    cur_ty = next_cur_ty;
                }

                if cur_ty != &dst_ty {
                    return Err(EngineError::InvalidAssumption(
                        "GEP destination type mismatch".into(),
                    ));
                }

                let pointer_new = self.parse_value(pointer, &Type::Pointer)?;
                Instruction::GEP {
                    src_pointee_type: src_ty,
                    dst_pointee_type: dst_ty,
                    pointer: pointer_new,
                    offset: offset_new,
                    indices: indices_new,
                    result: index.into(),
                }
            }
            // choice
            AdaptedInst::ITE {
                cond,
                then_value,
                else_value,
            } => {
                let cond_new = self.parse_value(cond, &Type::Bitvec { bits: 1 })?;
                let inst_ty = self.typing.convert(ty)?;
                let then_value_new = self.parse_value(then_value, &inst_ty)?;
                let else_value_new = self.parse_value(else_value, &inst_ty)?;
                Instruction::ITE {
                    cond: cond_new,
                    then_value: then_value_new,
                    else_value: else_value_new,
                    result: index.into(),
                }
            }
            AdaptedInst::Phi { options } => {
                let inst_ty = self.typing.convert(ty)?;
                let mut options_new = BTreeMap::new();
                for opt in options {
                    if !self.blocks.contains(&opt.block) {
                        return Err(EngineError::InvariantViolation(
                            "unknown incoming edge into phi node".into(),
                        ));
                    }
                    let value_new = self.parse_value(&opt.value, &inst_ty)?;
                    let label_new = opt.block.into();
                    match options_new.get(&label_new) {
                        None => (),
                        Some(existing) => {
                            // TODO(mengxu): LLVM IR may contain duplicated entries with the same label/value pair
                            if existing != &value_new {
                                return Err(EngineError::InvariantViolation(
                                    "duplicated edges into phi node with different values".into(),
                                ));
                            }
                        }
                    }
                    options_new.insert(label_new, value_new);
                }
                Instruction::Phi {
                    ty: inst_ty,
                    options: options_new,
                    result: index.into(),
                }
            }
            // aggregates
            AdaptedInst::GetValue {
                from_ty,
                aggregate,
                indices,
            } => {
                let src_ty = self.typing.convert(from_ty)?;
                let dst_ty = self.typing.convert(ty)?;

                let mut cur_ty = &src_ty;
                for idx in indices {
                    let next_cur_ty = match cur_ty {
                        Type::Struct { name: _, fields } => {
                            if *idx >= fields.len() {
                                return Err(EngineError::InvalidAssumption(
                                    "field number out of range".into(),
                                ));
                            }
                            fields.get(*idx).unwrap()
                        }
                        Type::Array { element, length } => {
                            if *idx >= *length {
                                return Err(EngineError::InvalidAssumption(
                                    "array index out of range".into(),
                                ));
                            }
                            element.as_ref()
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "Aggregate getter only applies to array and struct".into(),
                            ));
                        }
                    };
                    cur_ty = next_cur_ty;
                }

                if cur_ty != &dst_ty {
                    return Err(EngineError::InvalidAssumption(
                        "GetValue destination type mismatch".into(),
                    ));
                }

                let aggregate_new = self.parse_value(aggregate, &src_ty)?;
                Instruction::GetValue {
                    src_ty,
                    dst_ty,
                    aggregate: aggregate_new,
                    indices: indices.clone(),
                    result: index.into(),
                }
            }
            AdaptedInst::SetValue {
                aggregate,
                value,
                indices,
            } => {
                let src_ty = self.typing.convert(ty)?;
                let mut cur_ty = &src_ty;
                for idx in indices {
                    let next_cur_ty = match cur_ty {
                        Type::Struct { name: _, fields } => {
                            if *idx >= fields.len() {
                                return Err(EngineError::InvalidAssumption(
                                    "field number out of range".into(),
                                ));
                            }
                            fields.get(*idx).unwrap()
                        }
                        Type::Array { element, length } => {
                            if *idx >= *length {
                                return Err(EngineError::InvalidAssumption(
                                    "array index out of range".into(),
                                ));
                            }
                            element.as_ref()
                        }
                        _ => {
                            return Err(EngineError::InvalidAssumption(
                                "Aggregate getter only applies to array and struct".into(),
                            ));
                        }
                    };
                    cur_ty = next_cur_ty;
                }
                let dst_ty = cur_ty.clone();

                let aggregate_new = self.parse_value(aggregate, &src_ty)?;
                let value_new = self.parse_value(value, &dst_ty)?;
                Instruction::SetValue {
                    src_ty,
                    dst_ty,
                    aggregate: aggregate_new,
                    value: value_new,
                    indices: indices.clone(),
                    result: index.into(),
                }
            }
            AdaptedInst::GetElement { .. }
            | AdaptedInst::SetElement { .. }
            | AdaptedInst::ShuffleVector { .. } => {
                return Err(EngineError::NotSupportedYet(Unsupported::Vectorization));
            }
            // concurrency
            AdaptedInst::Fence { .. }
            | AdaptedInst::AtomicCmpXchg { .. }
            | AdaptedInst::AtomicRMW { .. } => {
                return Err(EngineError::NotSupportedYet(Unsupported::AtomicInstruction));
            }
            // terminators should never appear here
            AdaptedInst::Return { .. }
            | AdaptedInst::Branch { .. }
            | AdaptedInst::Switch { .. }
            | AdaptedInst::IndirectJump { .. }
            | AdaptedInst::Unreachable => {
                return Err(EngineError::InvariantViolation(
                    "malformed block with terminator instruction in the body".into(),
                ));
            }
        };
        Ok(item)
    }

    /// convert an instruction to a terminator
    pub fn parse_terminator(
        &mut self,
        inst: &adapter::instruction::Instruction,
    ) -> EngineResult<Terminator> {
        use adapter::instruction::Inst as AdaptedInst;
        use adapter::typing::Type as AdaptedType;

        // all terminator instructions have a void type
        if !matches!(inst.ty, AdaptedType::Void) {
            return Err(EngineError::InvalidAssumption(
                "all terminator instructions must have void type".into(),
            ));
        }

        let term = match &inst.repr {
            AdaptedInst::Return { value } => match (value, &self.ret) {
                (None, None) => Terminator::Return { val: None },
                (Some(_), None) | (None, Some(_)) => {
                    return Err(EngineError::InvariantViolation(
                        "return type mismatch".into(),
                    ));
                }
                (Some(val), Some(ty)) => {
                    let converted = self.parse_value(val, &ty.clone())?;
                    Terminator::Return {
                        val: Some(converted),
                    }
                }
            },
            AdaptedInst::Branch { cond, targets } => match cond {
                None => {
                    if targets.len() != 1 {
                        return Err(EngineError::InvalidAssumption(
                            "unconditional branch should have exactly one target".into(),
                        ));
                    }
                    let target = targets.first().unwrap();
                    if !self.blocks.contains(target) {
                        return Err(EngineError::InvalidAssumption(
                            "unconditional branch to unknown target".into(),
                        ));
                    }
                    Terminator::Goto {
                        target: target.into(),
                    }
                }
                Some(val) => {
                    let cond_new = self.parse_value(val, &Type::Bitvec { bits: 1 })?;
                    if targets.len() != 2 {
                        return Err(EngineError::InvalidAssumption(
                            "conditinal branch should have exactly two targets".into(),
                        ));
                    }
                    #[allow(clippy::get_first)] // for symmetry
                    let target_then = targets.get(0).unwrap();
                    if !self.blocks.contains(target_then) {
                        return Err(EngineError::InvalidAssumption(
                            "conditional branch to unknown then target".into(),
                        ));
                    }
                    let target_else = targets.get(1).unwrap();
                    if !self.blocks.contains(target_else) {
                        return Err(EngineError::InvalidAssumption(
                            "conditional branch to unknown else target".into(),
                        ));
                    }
                    Terminator::Branch {
                        cond: cond_new,
                        then_case: target_then.into(),
                        else_case: target_else.into(),
                    }
                }
            },
            AdaptedInst::Switch {
                cond,
                cond_ty,
                cases,
                default,
            } => {
                let cond_ty_new = self.typing.convert(cond_ty)?;
                if !matches!(cond_ty_new, Type::Bitvec { .. }) {
                    return Err(EngineError::InvalidAssumption(
                        "switch condition must be bitvec".into(),
                    ));
                }
                let cond_new = self.parse_value(cond, &cond_ty_new)?;

                let mut mapping = BTreeMap::new();
                for case in cases {
                    if !self.blocks.contains(&case.block) {
                        return Err(EngineError::InvalidAssumption(
                            "switch casing into an invalid block".into(),
                        ));
                    }

                    let case_val =
                        Constant::convert(&case.value, &cond_ty_new, self.typing, self.symbols)?;
                    let label_val = match case_val {
                        Constant::Bitvec {
                            bits: _,
                            value: label_val,
                        } => match label_val.to_u64() {
                            None => {
                                return Err(EngineError::InvalidAssumption(
                                    "switch casing label larger than u64".into(),
                                ));
                            }
                            Some(v) => v,
                        },
                        _ => {
                            return Err(EngineError::InvariantViolation(
                                "switch case is not a constant bitvec".into(),
                            ));
                        }
                    };
                    mapping.insert(label_val, case.block.into());
                }

                let default_new = match default {
                    None => None,
                    Some(label) => {
                        if !self.blocks.contains(label) {
                            return Err(EngineError::InvalidAssumption(
                                "switch default casing into an invalid block".into(),
                            ));
                        }
                        Some(label.into())
                    }
                };

                Terminator::Switch {
                    cond: cond_new,
                    cases: mapping,
                    default: default_new,
                }
            }
            AdaptedInst::IndirectJump { .. } => {
                return Err(EngineError::NotSupportedYet(Unsupported::AtomicInstruction));
            }
            AdaptedInst::Unreachable => Terminator::Unreachable,
            // explicitly list the rest of the instructions
            AdaptedInst::Alloca { .. }
            | AdaptedInst::Load { .. }
            | AdaptedInst::Store { .. }
            | AdaptedInst::VAArg { .. }
            | AdaptedInst::Intrinsic { .. }
            | AdaptedInst::CallDirect { .. }
            | AdaptedInst::CallIndirect { .. }
            | AdaptedInst::Asm { .. }
            | AdaptedInst::Unary { .. }
            | AdaptedInst::Binary { .. }
            | AdaptedInst::Compare { .. }
            | AdaptedInst::Cast { .. }
            | AdaptedInst::Freeze { .. }
            | AdaptedInst::GEP { .. }
            | AdaptedInst::ITE { .. }
            | AdaptedInst::Phi { .. }
            | AdaptedInst::GetValue { .. }
            | AdaptedInst::SetValue { .. }
            | AdaptedInst::GetElement { .. }
            | AdaptedInst::SetElement { .. }
            | AdaptedInst::ShuffleVector { .. }
            | AdaptedInst::Fence { .. }
            | AdaptedInst::AtomicCmpXchg { .. }
            | AdaptedInst::AtomicRMW { .. } => {
                return Err(EngineError::InvariantViolation(
                    "malformed block with non-terminator instruction".into(),
                ));
            }
        };
        Ok(term)
    }
}
