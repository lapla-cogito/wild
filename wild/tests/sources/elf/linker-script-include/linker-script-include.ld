INCLUDE linker-script-include-sub.ld

SECTIONS {
    . = 0x600000;
    .text : { *(.text .text.*) }
    .data : { *(.data .data.*) }
    .bss : { *(.bss .bss.*) }
}
