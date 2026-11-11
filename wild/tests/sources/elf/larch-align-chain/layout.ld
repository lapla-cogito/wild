ENTRY(_start)
SECTIONS {
    .text 0x10000 : { KEEP(*(.text)) }
}
