//#LinkArgs:-L $SRC_DIR -T ./linker-script-include.ld -z now
//#Object:runtime.c
//#ExpectSym:included_sym address=0xabcd1234

#include "../common/runtime.h"

void _start(void) { exit_syscall(42); }
