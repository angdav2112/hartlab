/* PolarFire playground: DDR at 0x80000000. Keep in sync with platforms/polarfire/MEMORY.md */
MEMORY
{
    RAM : ORIGIN = 0x80000000, LENGTH = 64K
}

_stack_size = 4K;
_stack_end = ORIGIN(RAM) + LENGTH(RAM);
_stack_start = _stack_end - _stack_size;

SECTIONS
{
    .text : {
        KEEP(*(.text.init))
        *(.text .text.*)
    } > RAM

    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > RAM

    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > RAM

    .bss (NOLOAD) : {
        _bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        _bss_end = .;
    } > RAM

    /DISCARD/ : {
        *(.eh_frame)
        *(.eh_frame_hdr)
    }
}

PROVIDE(_stack_end = _stack_end);
PROVIDE(_stack_start = _stack_start);
PROVIDE(_bss_start = _bss_start);
PROVIDE(_bss_end = _bss_end);
