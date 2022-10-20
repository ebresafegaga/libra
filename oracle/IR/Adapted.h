#ifndef LIBRA_IR_ADAPTED_H
#define LIBRA_IR_ADAPTED_H

#include "Shared/Deps.h"

namespace libra::ir::adapted {

/// A fully defined function
class Function {
public:
  /// name of the function
  std::string name_;

public:
  /// initialize from an LLVM function
  explicit Function(const llvm::Function &f);
};

/// Module, which is also an encapsulation of the whole context
class Module {
public:
  /// name of the module
  std::string name_;
  /// functions defined in this module, ordered by name
  std::map<std::string, Function> functions_;

public:
  /// initialize from an LLVM module
  explicit Module(const llvm::Module &m);
};

} // namespace libra::ir::adapted

#endif // LIBRA_IR_ADAPTED_H
