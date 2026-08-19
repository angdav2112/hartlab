# HartLab architecture (as-built)

Long-term shape, isolation flags, and MVP definition of done:
[`VISION.md`](VISION.md). Phases and leftover work: [`ROADMAP.md`](ROADMAP.md).

HartLab is a browser playground for PolarFire SoC GDB practice.

```
Browser (Vercel) --HTTPS/WSS--> Axum on a Hetzner Cloud VPS
                                      |
                                      +-- docker run --network=none
                                           hartlab-session
                                             agent + Renode + riscv64-elf-gdb
```

## Isolation

One hardened Docker container per session. No Firecracker, no dedicated
KVM host. Flags: `--network=none`, `--read-only`, `--cap-drop ALL`,
memory/cpu/pids limits, bind-mounted ELF only.

The browser never talks to Renode or GDB directly. The agent speaks a
fixed JSON schema (`crates/protocol`).

## PolarFire model

Renode `platforms/cpus/polarfire-soc.repl` plus `platforms/polarfire/playground.repl`
(four `Miscellaneous.LED`s on GPIO2 pins 16–19).

GDB stub: `machine StartGdbServer 3333 false`. Five RV64 harts appear as
GDB threads. Application firmware runs on **U54_1** (`mhartid == 1`).

## Status

Phase 0 in this repo: platform pack and a teaching Rust blinky. Control,
agent, and the session Dockerfile are **stubs**. Next work is the usable
public cut in [`ROADMAP.md`](ROADMAP.md): isolation gate → real session →
SPA → https://hartlab.vesperforge.org. Do not grow the website until
`cap-drop ALL` is proven.
