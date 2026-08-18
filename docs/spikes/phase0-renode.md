# Phase 0 — live Renode fidelity

Host: Apple M5 (darwin-arm64), 2026-08-18.
Tools: Renode 1.16.1 (`renode-1.16.1-dotnet.osx-arm64-portable.dmg`),
xPack GDB 16.3 (`riscv-none-elf-gcc 15.2.0-1`).

## Result

`./tests/fidelity/run-live.sh` **PASS**.

| Check | Result |
|-------|--------|
| GDB threads | 5 (`icicle.e51` … `icicle.u54_4`) |
| Break `rust_main` | hit on thread 2 (`icicle.u54_1`), `src/main.rs` |
| LED JSON | `leds: [true, false, false, false]` after continue |
| Halt freeze | jsonl line count unchanged after SIGINT |
| LED bind | `sysbus.gpio2.led1`…`led4` `StateChanged` works |

Renode log (abridged):

```
icicle: CPUs: ["icicle.e51", "icicle.u54_1", "icicle.u54_2",
               "icicle.u54_3", "icicle.u54_4"]
        were added to a new GDB server created on port :3333
sysbus: Loading block of 184 bytes length at 0x80000000.
```

GDB:

```
* 1    Thread 1 "icicle.e51"   0x80000000 in _start ()
  2    Thread 2 "icicle.u54_1" 0x80000000 in _start ()
  ...
Breakpoint 1 at 0x8000003e: file src/main.rs, line 97.
Thread 2 "icicle.u54_1" hit Breakpoint 1 ... rust_main
```

## Gotchas found

1. **`StartGdbServer 3333 false` is not enough.** e51 is `rv64imac`, U54s
   are `rv64gc`. Renode 1.16 requires `cpuCluster="all"` (or a single
   cluster). `all` gives five threads; xPack RISC-V GDB accepts both ISAs.
2. **GDB `continue` does not start virtual time.** Need `monitor start`
   first (`StartGdbServer … false`). SoftConsole-like teaching should
   document this.
3. **`--console` + stdin EOF disposes the machine.** The runner keeps
   stdin open and does not pass `--console`.
4. **`$ROOT` does not expand in `@$ROOT/path`.** Overlay files are
   repo-relative; the runner sets cwd to the repo root.
5. **IronPython 2 is ASCII-only** without an encoding cookie. No em-dashes
   in `playground.py`.
6. **`set startup-with-shell off` in a `-x` script aborts** this GDB
   (`No symbol table is loaded`). The agent still uses the denylist; the
   spike script does not send that command.
7. **Stock PolarFire translation-cache warning** on this host: requested
   512 MiB, clamped to 128 MiB. Not an RSS measurement of DDR.

## Docker / cap-drop ALL

This Mac has no Docker Engine. Lock-down flags are exercised on
`ubuntu-latest` by `.github/workflows/fidelity.yml` via
`tests/fidelity/run-docker.sh`:

`--network=none --read-only --cap-drop ALL --memory 1536m --cpus 1
--pids-limit 64 --security-opt no-new-privileges` plus tmpfs `/tmp` and
`/home/session`.

Until that job is green, treat **cap-drop ALL: pending CI**.

## RSS

Not measured here (no cgroup on macOS for the host spike). CI docker job
prints image size. Still do the playground-DDR shrink spike before
promising 4 sessions on 16 GB.

## How to reproduce

```bash
./scripts/build-blinky.sh
./scripts/fetch-host-tools.sh   # ~/.cache/hartlab/tools
./tests/fidelity/run-live.sh
```
