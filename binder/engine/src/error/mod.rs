use std::error::Error;
use std::fmt::{Display, Formatter};

/// A list of operations not supported
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Unsupported {
    ModuleLevelAssembly,
    InlineAssembly,
    GlobalAlias,
    GlobalMarker,
    FloatingPointOrdering,
    VectorOfPointers,
    ScalableVector,
    VectorBitcast,
    VariadicArguments,
    ArchSpecificExtension,
    TypedPointer,
    ThreadLocalStorage,
    WeakGlobalVariable,
    WeakFunction,
    ExternGlobalVariable,
    ExternFunction,
    PointerAddressSpace,
    OutOfBoundConstantGEP,
    InterfaceResolver,
    AnonymousFunction,
    IndirectJump,
    IntrinsicsExperimentalGC,
    IntrinsicsPreAllocated,
    AtomicInstruction,
    ExceptionHandling,
    MetadataSystem,
}

impl Display for Unsupported {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModuleLevelAssembly => {
                write!(f, "module-level assembly")
            }
            Self::InlineAssembly => {
                write!(f, "inline assembly")
            }
            Self::GlobalAlias => {
                write!(f, "global alias")
            }
            Self::GlobalMarker => {
                write!(f, "markers for global values")
            }
            Self::FloatingPointOrdering => {
                write!(f, "floating point ordered comparison")
            }
            Self::VectorOfPointers => {
                write!(f, "vector of pointers")
            }
            Self::ScalableVector => {
                write!(f, "scalable vector of non-fixed size")
            }
            Self::VectorBitcast => {
                write!(f, "bitcast among vector and scalar")
            }
            Self::VariadicArguments => {
                write!(f, "variadic arguments")
            }
            Self::ArchSpecificExtension => {
                write!(f, "architecture-specific extension")
            }
            Self::TypedPointer => {
                write!(f, "typed pointer")
            }
            Self::ThreadLocalStorage => {
                write!(f, "thread-local storage")
            }
            Self::WeakGlobalVariable => {
                write!(f, "weak definition for global variable")
            }
            Self::WeakFunction => {
                write!(f, "weak definition for function")
            }
            Self::ExternGlobalVariable => {
                write!(f, "global variable externally initialized")
            }
            Self::ExternFunction => {
                write!(f, "function externally defined")
            }
            Self::PointerAddressSpace => {
                write!(f, "address space of a pointer")
            }
            Self::OutOfBoundConstantGEP => {
                write!(f, "intentional out-of-bound GEP on constant")
            }
            Self::InterfaceResolver => {
                write!(f, "load-time interface resolving")
            }
            Self::AnonymousFunction => {
                write!(f, "anonymous function")
            }
            Self::IndirectJump => {
                write!(f, "indirect jump (e.g., through register)")
            }
            Self::IntrinsicsExperimentalGC => {
                write!(f, "llvm.experimental.gc.*")
            }
            Self::IntrinsicsPreAllocated => {
                write!(f, "llvm.call.preallocated.*")
            }
            Self::AtomicInstruction => {
                write!(f, "atomic instruction")
            }
            Self::ExceptionHandling => {
                write!(f, "exception handling")
            }
            Self::MetadataSystem => {
                write!(f, "metadata system")
            }
        }
    }
}

/// A custom error message for the analysis engine
#[derive(Debug, Clone)]
pub enum EngineError {
    /// Error during the compilation of the input
    CompilationError(String),
    /// Error during the loading of a compiled LLVM module
    LLVMLoadingError(String),
    /// Invalid assumption made about the program
    InvalidAssumption(String),
    /// Operation not supported yet
    NotSupportedYet(Unsupported),
    /// Invariant violation
    InvariantViolation(String),
}

pub type EngineResult<T> = Result<T, EngineError>;

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompilationError(msg) => {
                write!(f, "[libra::compilation] {}", msg)
            }
            Self::LLVMLoadingError(msg) => {
                write!(f, "[libra::loading] {}", msg)
            }
            Self::InvalidAssumption(msg) => {
                write!(f, "[libra::assumption] {}", msg)
            }
            Self::NotSupportedYet(item) => {
                write!(f, "[libra::unsupported] {}", item)
            }
            Self::InvariantViolation(msg) => {
                write!(f, "[libra::invariant] {}", msg)
            }
        }
    }
}

impl Error for EngineError {}
