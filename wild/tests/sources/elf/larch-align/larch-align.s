/*
//#AbstractConfig:base
//#Arch:loongarch64
//#LinkerScript:layout.ld
//#LinkArgs:-nostdlib -static -no-pie
//#DiffEnabled:false
//#ReferenceLinkers:bfd,lld
// The assembler emits the worst-case NOP sled. BFD and LLD delete the excess even with
// --no-relax.
//#ExpectSym:after section=".text",offset-in-section=16,alignment=16
//#ExpectSym:after2 section=".text",offset-in-section=32,alignment=16

//#Config:relax:base

//#Config:no-relax:base
//#LinkArgs:--no-relax
*/

.text
.globl _start
.type _start, @function
_start:
    addi.w  $r4, $r0, 42
    addi.w  $r11, $r0, 93
    syscall 0

.p2align 4
.globl after
after:
    nop

.p2align 4
.globl after2
after2:
    nop
.size _start, .-_start
