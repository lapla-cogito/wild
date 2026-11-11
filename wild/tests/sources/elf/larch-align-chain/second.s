.section .text, "ax", @progbits
.globl second
second:
    .reloc ., R_LARCH_ALIGN, 0xc
    nop
    nop
    nop
.globl tail
tail:
    nop
