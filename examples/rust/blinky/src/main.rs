//! PolarFire Icicle blinky for HartLab.
//!
//! Runs on U54_1 (`mhartid == 1`). Other harts WFI.
//! Toggles GPIO2 pin 16 (LED1) so GDB halt freezes a visible LED.

#![no_std]
#![no_main]

use core::ptr::{read_volatile, write_volatile};

const SYSREG: usize = 0x2000_2000;
const SUBBLK_CLOCK_CR: usize = SYSREG + 0x84;
const SOFT_RESET_CR: usize = SYSREG + 0x88;
const GPIO2_CLOCK_BIT: u32 = 1 << 22;

const GPIO2: usize = 0x2012_2000;
const GPIO_EN_OUT: u32 = 1 << 0;
const GPIO_EN_OUT_BUF: u32 = 1 << 2;
const GPIO_OUTP: usize = GPIO2 + 0x88;
const LED1_PIN: u32 = 16;
const PERIOD_TICKS: u64 = 500_000; // ~0.5 s at CLINT 1 MHz

const CLINT_MTIME: usize = 0x0200_BFF8;

core::arch::global_asm!(
    r#"
    .section .text.init,"ax"
    .global _start
_start:
    csrr t0, mhartid
    li   t1, 1
    bne  t0, t1, 3f

    la   t0, _bss_start
    la   t1, _bss_end
1:
    bgeu t0, t1, 2f
    sd   zero, 0(t0)
    addi t0, t0, 8
    j    1b
2:
    la   sp, _stack_end
    jal  rust_main
3:
    wfi
    j    3b
    "#
);

fn mmio_read(addr: usize) -> u32 {
    unsafe { read_volatile(addr as *const u32) }
}

fn mmio_write(addr: usize, value: u32) {
    unsafe { write_volatile(addr as *mut u32, value) }
}

fn mtime() -> u64 {
    unsafe { read_volatile(CLINT_MTIME as *const u64) }
}

fn delay_ticks(ticks: u64) {
    let start = mtime();
    while mtime().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

fn enable_gpio2() {
    let clocks = mmio_read(SUBBLK_CLOCK_CR) | GPIO2_CLOCK_BIT;
    mmio_write(SUBBLK_CLOCK_CR, clocks);
    let reset = mmio_read(SOFT_RESET_CR) & !GPIO2_CLOCK_BIT;
    mmio_write(SOFT_RESET_CR, reset);
}

fn gpio_ctrl(pin: u32) -> usize {
    GPIO2 + 4 * pin as usize
}

fn configure_led_output(pin: u32) {
    mmio_write(gpio_ctrl(pin), GPIO_EN_OUT | GPIO_EN_OUT_BUF);
}

fn set_led(pin: u32, on: bool) {
    let mut out = mmio_read(GPIO_OUTP);
    if on {
        out |= 1 << pin;
    } else {
        out &= !(1 << pin);
    }
    mmio_write(GPIO_OUTP, out);
}

/// Application entry on U54_1. GDB can break here.
#[inline(never)]
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    enable_gpio2();
    configure_led_output(LED1_PIN);
    let mut on = false;
    loop {
        on = !on;
        set_led(LED1_PIN, on);
        delay_ticks(PERIOD_TICKS);
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
