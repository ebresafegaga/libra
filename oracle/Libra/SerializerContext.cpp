#include "Serializer.h"

namespace libra {

BasicBlock *dummy_block = nullptr;
Function *dummy_function = nullptr;
Instruction *dummy_instruction = nullptr;

void prepare_for_serialization(Module &module) {
  auto &ctxt = module.getContext();
  dummy_function =
      Function::Create(FunctionType::get(Type::getVoidTy(ctxt), false),
                       GlobalValue::LinkageTypes::InternalLinkage, "", &module);
  dummy_block = BasicBlock::Create(ctxt, "", dummy_function);
  dummy_instruction = new UnreachableInst(ctxt, dummy_block);
}

void FunctionSerializationContext::add_block(const llvm::BasicBlock &block) {
  auto index = block_labels_.size();
  auto res = block_labels_.emplace(&block, index);
  assert(res.second);
}

void FunctionSerializationContext::add_instruction(
    const llvm::Instruction &inst) {
  auto index = inst_labels_.size();
  auto res = inst_labels_.emplace(&inst, index);
  assert(res.second);
}

void FunctionSerializationContext::add_argument(const llvm::Argument &arg) {
  auto index = arg_labels_.size();
  auto res = arg_labels_.emplace(&arg, index);
  assert(res.second);
}

uint64_t
FunctionSerializationContext::get_block(const BasicBlock &block) const {
  return block_labels_.at(&block);
}

uint64_t
FunctionSerializationContext::get_instruction(const Instruction &inst) const {
  return inst_labels_.at(&inst);
}

uint64_t FunctionSerializationContext::get_argument(const Argument &arg) const {
  return arg_labels_.at(&arg);
}

} // namespace libra