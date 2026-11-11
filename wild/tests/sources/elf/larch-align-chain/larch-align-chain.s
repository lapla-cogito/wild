/*
//#AbstractConfig:base
//#Arch:loongarch64
//#Object:second.s
//#LinkerScript:layout.ld
//#LinkArgs:-nostdlib -static -no-pie
//#DiffEnabled:false
//#ReferenceLinkers:bfd,lld
// `.reloc` does not raise sh_addralign. The second sled's address depends on how many NOPs the
// first sled keeps, so that count has to be recomputed after the first section shrinks.
//#ExpectSym:mid section=".text",offset-in-section=16,alignment=16
//#ExpectSym:tail section=".text",offset-in-section=32,alignment=16

//#Config:relax:base

//#Config:no-relax:base
//#LinkArgs:--no-relax
*/

.section .text, "ax", @progbits
.globl _start
_start:
    addi.w  $r4, $r0, 42
    addi.w  $r11, $r0, 93
    syscall 0
    .reloc ., R_LARCH_ALIGN, 0xc
    nop
    nop
    nop
.globl mid
mid:
    nop
