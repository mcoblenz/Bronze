#include <stdio.h>

#include "Bronze.h"


struct LLVMStackEntry *get_llvm_gc_root_chain() {
    return llvm_gc_root_chain;
}