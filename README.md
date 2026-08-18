# HartLab

Browser playground for PolarFire SoC GDB practice. Users load curated or
uploaded RISC-V firmware onto a Renode PolarFire model, debug it with real
GDB, and watch LEDs and UART update live.

**Status:** Phase 0. PolarFire pack + teaching blinky are real.
Control/agent/Docker are stubs. Public URL is **not** live yet.

**Plan (do not add another):** [`docs/VISION.md`](docs/VISION.md) (product) +
[`docs/ROADMAP.md`](docs/ROADMAP.md) (usable cut for
https://hartlab.vesperforge.org). Index: [`docs/README.md`](docs/README.md).
Branches: [`CONTRIBUTING.md`](CONTRIBUTING.md).

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

```bash
./scripts/build-blinky.sh
./scripts/fetch-host-tools.sh    # pinned Renode 1.16.1 + xPack GDB
./tests/fidelity/run-live.sh     # 5 threads, rust_main, LED freeze
```

Tools land in `~/.cache/hartlab/tools`, not in git. Results:
`docs/spikes/phase0-renode.md`.

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
