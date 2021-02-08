#include <stdio.h>

#include "Bronze.h"


// Must initialize this global here so that it gets defined as a regular symbol, not a "common" symbol.
struct LLVMStackEntry *llvm_gc_root_chain_bronze_ref = NULL; // Exported by this library

void bronze_init() {
    printf("Initializing llvm_gc_root_chain_bronze_ref to %p\n", llvm_gc_root_chain);
    llvm_gc_root_chain_bronze_ref = llvm_gc_root_chain;
}