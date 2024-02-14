# General
set(CMAKE_BUILD_TYPE Release CACHE STRING "")

# Stage 1 setup
set(CLANG_ENABLE_BOOTSTRAP ON CACHE BOOL "")
set(CLANG_BOOTSTRAP_TARGETS
    clang
    check-all
    check-llvm
    check-clang
    test-suite
    stage3
    stage3-clang
    stage3-check-all
    stage3-check-llvm
    stage3-check-clang
    stage3-test-suite
    stage3-install CACHE STRING "")

# Stage 1: build clang and lld with system cc
#          the new clang has -fuse-ld=lld support
set(STAGE1_PROJECTS "clang;lld")
set(STAGE1_RUNTIMES "")

set(LLVM_TARGETS_TO_BUILD Native CACHE STRING "")
set(LLVM_ENABLE_PROJECTS ${STAGE1_PROJECTS} CACHE STRING "")
set(LLVM_ENABLE_RUNTIMES ${STAGE1_RUNTIMES} CACHE STRING "")

# Stage 2 setup
set(BOOTSTRAP_CLANG_ENABLE_BOOTSTRAP ON CACHE STRING "")
set(BOOTSTRAP_CLANG_BOOTSTRAP_TARGETS
    clang
    check-all
    check-llvm
    check-clang
    test-suite CACHE STRING "")

# Stage 2: build basic llvm tools with stage1-clang -flto=full
#          the new clang has -fuse-ld=lld and -stdlib=libc++ support
set(STAGE2_PROJECTS "clang;libc;lld")
set(STAGE2_RUNTIMES "compiler-rt;libcxx;libcxxabi;libunwind")

set(BOOTSTRAP_LLVM_TARGETS_TO_BUILD Native CACHE STRING "")
set(BOOTSTRAP_LLVM_ENABLE_PROJECTS ${STAGE2_PROJECTS} CACHE STRING "")
set(BOOTSTRAP_LLVM_ENABLE_RUNTIMES ${STAGE2_RUNTIMES} CACHE STRING "")
set(BOOTSTRAP_LLVM_ENABLE_LLD ON CACHE BOOL "")
set(BOOTSTRAP_LLVM_ENABLE_LTO "Full" CACHE STRING "")
set(BOOTSTRAP_CLANG_DEFAULT_CXX_STDLIB "libc++" CACHE STRING "")
set(BOOTSTRAP_CLANG_DEFAULT_RTLIB "compiler-rt" CACHE STRING "")
set(BOOTSTRAP_CLANG_DEFAULT_UNWINDLIB "libunwind" CACHE STRING "")
set(BOOTSTRAP_LIBC_ENABLE_USE_BY_CLANG ON CACHE BOOL "")

# Stage 3: build extra llvm tools with stage2-clang -flto=full -stdlib=libc++
#          the new clang has -fuse-ld=lld and -stdlib=libc++ support
set(STAGE3_PROJECTS "clang;clang-tools-extra;libc;lld;lldb;polly")
set(STAGE3_RUNTIMES "compiler-rt;libcxx;libcxxabi;libunwind")

set(BOOTSTRAP_BOOTSTRAP_LLVM_TARGETS_TO_BUILD Native CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_PROJECTS ${STAGE3_PROJECTS} CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_RUNTIMES ${STAGE3_RUNTIMES} CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_LLD ON CACHE BOOL "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_LTO "Full" CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_RTTI ON CACHE BOOL "")
set(BOOTSTRAP_BOOTSTRAP_LLVM_ENABLE_LIBCXX ON CACHE BOOL "")
set(BOOTSTRAP_BOOTSTRAP_CLANG_DEFAULT_CXX_STDLIB "libc++" CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_CLANG_DEFAULT_RTLIB "compiler-rt" CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_CLANG_DEFAULT_UNWINDLIB "libunwind" CACHE STRING "")
set(BOOTSTRAP_BOOTSTRAP_LIBC_ENABLE_USE_BY_CLANG ON CACHE BOOL "")
