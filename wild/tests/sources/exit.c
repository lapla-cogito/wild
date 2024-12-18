#include "exit.h"

#include <inttypes.h>
#include <sys/types.h>

#if defined(__x86_64__)
void exit_syscall(int exit_code) {
    register int64_t rax __asm__ ("rax") = 60;
    register int rdi __asm__ ("rdi") = exit_code;
    __asm__ __volatile__ (
        "syscall"
        : "+r" (rax)
        : "r" (rdi)
        : "rcx", "r11", "memory"
    );
}
#elif defined(__aarch64__)
void exit_syscall(int exit_code) {
    register long w8 __asm__("w8") = 93;
    register long x0 __asm__("x0") = exit_code;
    __asm__ __volatile__(
        "svc 0"
        : "=r"(x0)
        : "r"(w8)
        : "cc", "memory");
}
#endif
