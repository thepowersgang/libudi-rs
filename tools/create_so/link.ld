
SECTIONS {
    . = 0x10000;
    .text : {
        *(.text .text.*)
    }
    .rodata : {
        *(.rodata .rodata.*)
        UDIPROPS_start = .;
        *(.udiprops)
        UDIPROPS_end = .;
    }
    .data : {
        *(.data .data.*)
    }
}