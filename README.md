# HartLab

Browser playground for PolarFire SoC GDB practice. Users load curated or
uploaded RISC-V firmware onto a Renode PolarFire model, debug it with real
GDB, and watch LEDs and UART update live.

**Status:** Phase 0 — platform pack + firmware examples + host control-plane
skeleton. Isolation is hardened Docker per session (see `docs/architecture.md`).
Firecracker / dedicated KVM hosts are out of scope.

## Layout

| Path | What |
|------|------|
| `platforms/polarfire/` | Renode playground overlay (LEDs on GPIO2 16–19) |
| `examples/rust/` | `no_std` firmware (blinky first) |
| `examples/ada/` | Parallel Ada sources (prebuilt ELF OK) |
| `apps/control/` | Axum session host |
| `apps/agent/` | In-container agent (Renode + GDB) |
| `crates/protocol/` | Shared JSON protocol + ELF accept list |
| `infra/docker/` | Session image |

## Quick start (firmware)

```bash
rustup target add riscv64gc-unknown-none-elf
./scripts/build-blinky.sh
```

The ELF lands at `examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky`.

## Quick start (Renode fidelity)

Needs Renode on `PATH` (portable .NET package is fine):

```bash
./tests/fidelity/run-blinky.sh
```

That script loads `platforms/polarfire/playground.resc`, starts the GDB stub,
and prints the hart / LED contract. It does not require the website.

## Quick start (host crates)

```bash
cargo test --workspace
cargo run -p hartlab-control
```

## Contract (firmware authors)

- Load address: `0x80000000` (DDR)
- Application hart: **U54_1** (`mhartid == 1`). Other harts must WFI.
- LEDs: MSS GPIO2 pins 16–19, active-high (Icicle LED1–LED4)
- UART: MMUART1 at `0x20100000` (later examples)

Details: `platforms/polarfire/MEMORY.md` and `docs/gdb-polarfire.md`.
