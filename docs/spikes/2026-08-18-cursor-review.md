# Spike note — Cursor review (2026-08-18)

External review of day-one `hartlab`. Kept here so the tree stays aligned
with the spec instead of with the stubs.

## Keep

- `docs/VISION.md` / `docs/ROADMAP.md` as the product definition
- Firmware/platform contract (U54_1, GPIO2 pin 16, e51-PC-copy)
- Threat model: assume Renode/GDB can be crashed; no net, no shell PTY

## Do not treat as spike progress

- `apps/control` / `apps/agent` — typed placeholders
- Ada `blinky.adb` — not a second gallery ELF
- Session Dockerfile — does not install Renode or GDB
- Isolation flags in the vision — documented, not demonstrated

## Fixed in-tree after the review

- Blinky release profile keeps DWARF (`debug = 2`, no LTO)
- `ENTRY(_start)` in `memory.x`
- GDB filter unwraps `-interpreter-exec console "…"`, default-deny
- ELF gate requires a PT_LOAD covering `0x80000000`, rejects 0 phdrs / ET_DYN
- Control stub no longer increments a session counter or advertises `/ws`
- Binds `127.0.0.1` by default; uses the `base64` crate
- Agent no longer depends on tokio or a no-op `/run/agent` path
- `playground.py` fails loudly if an LED has no `StateChanged`
- `infra/docker/seccomp-session.json` exists

## Still the Phase 0 exit (not done)

1. Live Renode: `info threads` == 5, break `rust_main`, LED JSON, halt freeze
2. Same under `--cap-drop ALL`; record RSS
3. Shrink DDR in the overlay if RSS forbids 4 sessions on 16 GB
4. Session image that actually runs Renode + GDB + agent

Do not grow Axum or add a second SoC until those are true.
