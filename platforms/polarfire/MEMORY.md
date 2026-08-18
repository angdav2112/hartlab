# PolarFire playground memory and I/O map

Used by every HartLab firmware example.

Firmware for this playground must match this contract so the same `.resc`
works for Rust, Ada, and uploaded ELFs.

## Load / RAM

| Region | Address | Size in playground | Notes |
|--------|---------|--------------------|--------|
| DDR (cached) | `0x80000000` | 64 KiB used by examples; SoC model still maps the stock PolarFire window | Put `.text` / `.data` / stack here |
| LIM | `0x08000000` | unused by examples | SoftConsole sometimes uses this; we do not |

Entry: `_start` at the ELF entry point. Renode `LoadELF` sets **e51** `PC`.
The playground reset macro copies that PC onto **u54_1** and parks u54_2–4.
Firmware must still check `mhartid` and only run the app on hart 1.

## Harts

| Hart | `mhartid` | Renode name | Playground role |
|------|-----------|-------------|-----------------|
| E51 | 0 | `e51` | WFI (monitor core) |
| U54_1 | 1 | `u54_1` | **Application** |
| U54_2 | 2 | `u54_2` | Halted |
| U54_3 | 3 | `u54_3` | Halted |
| U54_4 | 4 | `u54_4` | Halted |

GDB: `info threads` then `thread 2` is U54_1 on a typical 1-based listing
(thread 1 = e51). Confirm with `info registers mhartid`.

## GPIO / LEDs (Icicle)

GPIO2 base: `0x20122000` (`gpio2` in Renode).

| LED | Pin | Config reg | Output bit |
|-----|-----|------------|------------|
| LED1 | 16 | `base + 0x4*16` | `OUTP` bit 16 |
| LED2 | 17 | `base + 0x4*17` | `OUTP` bit 17 |
| LED3 | 18 | `base + 0x4*18` | `OUTP` bit 18 |
| LED4 | 19 | `base + 0x4*19` | `OUTP` bit 19 |

Per-pin config (`MPFS_GPIO_CTRL`):

- bit 0 `EN_OUT`
- bit 1 `EN_IN`
- bit 2 `EN_OUT_BUF`

Output-only: write `EN_OUT | EN_OUT_BUF` (`0x5`).

Bank registers:

- `IRQ`  `base + 0x80`
- `INP`  `base + 0x84`
- `OUTP` `base + 0x88`

LEDs are **active-high**.

Before touching GPIO2, enable the block in SYSREG (`0x20002000`):

- `SUBBLK_CLOCK_CR` at `+0x84`, set bit 22 (`GPIO2`)
- `SOFT_RESET_CR` at `+0x88`, clear bit 22 (`GPIO2`)

## UART (later examples)

MMUART1 (NS16550-ish, wide registers): `0x20100000`.
Renode name: `mmuart1`. IRQ via PLIC; for polled TX, write the THR.

## CLINT

`0x02000000`, 1 MHz in the stock PolarFire `.repl`.
`mtime` at `0x0200BFF8`. Useful for delays without a busy-spin that GDB
cannot make sense of — still fine to busy-wait on `mtime`.

## What uploaded ELFs must satisfy

- ELFCLASS64, `EM_RISCV` (243), no `PT_INTERP`
- Linked at `0x80000000` (or at least with a loadable segment there)
- Size ≤ 4 MiB
- Prefer DWARF so the source pane can highlight. Curated Rust blinky is
  built with `debug = 2` and no LTO for that reason.
