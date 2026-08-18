# HartLab — vision

**Product:** Browser playground for authentic PolarFire SoC GDB practice, with
live board visualization and no local install.

**Repo:** [angdav2112/hartlab](https://github.com/angdav2112/hartlab)

**Operator model:** One experienced developer. Marketing/docs on Vercel.
Interactive sessions on a **Hetzner Cloud VPS** (no dedicated/bare metal, no
KVM required).

**Flagship target:** Microchip PolarFire SoC (Icicle-style), 5 RISC-V harts,
Renode as the execution engine.

**Isolation:** **Hardened Docker per session** (gVisor later if a stronger
userland sandbox is wanted). Firecracker / dedicated servers are out of scope
unless we reopen that decision.

How we get there: [`ROADMAP.md`](ROADMAP.md). How it is built today:
[`architecture.md`](architecture.md).

---

## What it should feel like

A stranger opens a website, clicks Launch, and is in a SoftConsole-like GDB
session against a PolarFire model — with LEDs that actually move.

- Landing page: example gallery (Rust and Ada), docs, **Launch Playground**.
- Session start: PolarFire Renode model (default) + a curated example or a
  size-limited ELF upload. Source upload is out of scope on the public lab.
- One workspace page:
  - Large live board (LEDs first; gauges later). The graphic **freezes when
    the debugger is stopped** (Renode virtual time stops on halt).
  - Guided toolbar: Continue, Step Over/Into/Out, Reset Halt, quick breakpoints.
  - Register / memory inspector.
  - Toggleable full GDB terminal (xterm.js) talking to a real
    `riscv64-unknown-elf-gdb`.
  - Source viewer with current-line highlight, UART console, hart selector,
    session TTL.
- Sessions are short-lived and destroyed automatically.

This is a general embedded-systems lab, not an avionics product. PolarFire is
the flagship because multi-hart GDB is the skill we want people to practice.

---

## Two deployables

1. **Website (Vercel)** — landing, gallery, docs, playground SPA (`apps/web`).
2. **Session host (Hetzner Cloud)** — Rust/Axum starts **one Docker container
   per session**. Inside: Renode + `riscv64-unknown-elf-gdb` + a tiny agent.
   **`network=none`.** Host↔container is a Unix socket (or stdio). Never a
   public port on the container.

Docker on Cloud replaces Firecracker on bare metal. We give up hardware VM
isolation and compensate with a tight container profile and no browser shell.

---

## Architecture

```
                         ┌──────────────────────────────────┐
                         │  Vercel                          │
                         │  apps/web                        │
                         │  landing, docs, gallery, SPA     │
                         └───────────────┬──────────────────┘
                                         │ HTTPS
                                         │ POST /v1/sessions
                                         │ WSS  /v1/sessions/:id/ws
                         ┌───────────────▼──────────────────┐
                         │  Hetzner Cloud VPS               │
                         │  Caddy (TLS) → hartlab-control   │
                         │                                  │
                         │  session manager                 │
                         │  elf validator                   │
                         │  unix-socket proxy (per session) │
                         │  docker runner (bollard)         │
                         └───────────────┬──────────────────┘
                                         │ docker run
                                         │ network=none
                                         │ ro root + bind ELF
                         ┌───────────────▼──────────────────┐
                         │  hartlab-session                 │
                         │  (no net, cap-drop ALL)          │
                         │                                  │
                         │  agent ──┬── Renode (PolarFire)  │
                         │          │    GDB stub :3333     │
                         │          │    LED/UART hooks     │
                         │          └── riscv64-elf-gdb MI  │
                         └──────────────────────────────────┘
```

The website never talks to Renode or GDB directly.

**Session flow**

1. Browser `POST /v1/sessions { target:"polarfire", example:"blinky-rust" }`.
   Control plane checks concurrency + rate limit, writes a session dir,
   `docker run`.
2. Agent starts Renode with `platforms/polarfire/playground.resc`, loads the
   ELF, `StartGdbServer 3333 false`, parks unused harts, starts GDB MI,
   `target remote :3333`. Sends `{type:"ready"}`.
3. One WebSocket, two logical channels: `gdb` and `hw`.
4. LED/UART hooks → agent `hw` frames → board paints.
5. Step/continue is GDB/MI. Halt freezes virtual time and the LEDs.
6. TTL, idle, or tab close → `docker rm -f` + delete session dir.

---

## Workspace UX

Desktop-first (1280px); stack on mobile.

```
┌────────────────────────────────────────────┬─────────────────────┐
│  Board visualization (hero)                │  Debug toolbar      │
│  Icicle-ish SVG, 4 LEDs, UART activity     │  Cont / Step / Out  │
│  “Paused” frost overlay when halted        │  Reset Halt         │
│                                            │  Hart selector      │
├──────────────────────────┬─────────────────┤  Registers          │
│  Source / disassembly    │  UART console   │  Memory peek        │
├──────────────────────────┴─────────────────┤                     │
│  GDB terminal (xterm, collapsed by default)│  Session: 12:04 left│
└────────────────────────────────────────────┴─────────────────────┘
```

First-run card: “Type `info threads`, then `thread 2`, break on `rust_main`,
`continue`, watch LED1.”

- Curated examples: ship sources; GDB `fullname` maps into `/opt/examples/…`.
- Uploads: DWARF if present, else disassembly + current PC.
- Hart selector is `-thread-info` / `-thread-select` with e51 / U54_n labels.

---

## Stack

| Layer | Choice | Why |
|-------|--------|-----|
| Marketing + playground UI | Next.js (or Astro + React island) on Vercel | Existing TS/Vercel workflow. xterm.js + live board is a SPA. |
| Terminal | xterm.js + `@xterm/addon-fit` | Own attach, not the insecure demo. |
| Board (MVP) | SVG + CSS glow on 4 LEDs | Fast. Richer art later. |
| Control plane | Rust, Axum, tokio, `bollard` | One binary. No shell-out to `docker`. |
| Isolation | Hardened Docker (`runc`) | Runs on Hetzner Cloud. No KVM. |
| Later sandbox | gVisor `runsc` | Optional. Still no bare metal. |
| Session agent | Tiny Rust static binary | Auditable. Supervises Renode + GDB. |
| Sim | Renode portable/.NET, pinned | PolarFire + multi-hart GDB is first-class. |
| GDB | `riscv64-unknown-elf-gdb`, pinned | Real MI. |
| Firmware CI | GitHub Actions: rustc + Alire | Attach ELFs; session image has no compiler. |
| TLS | Caddy on the VPS | `play.<domain>`. |
| Persistence | None for MVP | Sessions are ephemeral. |
| Orchestration | systemd + Docker, one box | No k8s. |

**Not for v1:** Firecracker, Kata, compiling user source, QEMU, in-browser GDB,
exposing the Renode Monitor, Kubernetes, running sessions on Vercel.

---

## Isolation

Every session:

```
docker run -d --name hartlab-<id> \
  --runtime=runc \
  --network=none \
  --read-only \
  --tmpfs /tmp:rw,noexec,nosuid,size=16m \
  --tmpfs /home/session:rw,noexec,nosuid,size=8m \
  --mount type=bind,src=/var/lib/hartlab/sessions/<id>/firmware.elf,dst=/firmware.elf,ro \
  --mount type=bind,src=/var/lib/hartlab/sessions/<id>/sock,dst=/run/agent \
  --user 10000:10000 \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  --security-opt seccomp=infra/docker/seccomp-session.json \
  --pids-limit 64 \
  --memory 1536m --memory-swap 1536m \
  --cpus 1 \
  --ulimit nproc=64 --ulimit nofile=256 --ulimit fsize=33554432 \
  --label app=hartlab --label session=<id> \
  hartlab-session:<pin>
```

- No virtio-equivalent network. No published ports. GDB stub is
  `127.0.0.1:3333` **inside** the container.
- Axum is not root. `docker.sock` is not in the session image.
- ELF allowlist (`hartlab_protocol::validate_elf`) before the bind mount:
  ELFCLASS64, `EM_RISCV`, ET_EXEC, ≤ 4 MiB, no `PT_INTERP`.
- 15 min hard TTL, 5 min idle, 1 session per IP, `max_concurrent = 4` on
  8 vCPU / 16 GB.
- Destroy is idempotent (`docker rm -f` + session dir). Label
  `app=hartlab` so a crashed control plane can still reap.
- GDB: one process, `--interpreter=mi3`. Typed CLI goes through
  `-interpreter-exec console`. No `/bin/sh` PTY. Agent denylist: `shell`,
  `pipe`, `source`, `python`, `make`, `cd`, `target`, `run`, `file` (after
  load), `dump`, …
- Do not start Renode’s visualization HTTP server or expose Monitor.

Budget per session: ~0.6–1.1 GB RAM, 1 CPU. On 8 vCPU / 16 GB, CPU saturates
before RAM. Do not promise Linux-on-Icicle on one Cloud box.

More: [`security.md`](security.md), [`runbook.md`](runbook.md).

---

## PolarFire contract

Upstream Renode: `platforms/cpus/polarfire-soc.repl`. Do **not** use
`icicle-kit.resc` (that boots HSS/U-Boot/Linux).

| Hart | `mhartid` | Role |
|------|-----------|------|
| e51 | 0 | Monitor; firmware WFI |
| u54_1 | 1 | **Application** |
| u54_2…4 | 2–4 | Halted |

`LoadELF` / GDB `load` programs **e51** only. Reset macro copies `e51.PC` onto
`u54_1`. Firmware still checks `mhartid`.

Icicle LEDs: GPIO2 (`0x20122000`) pins 16–19, active-high. UART for later
examples: MMUART1 `0x20100000`. Load address: `0x80000000`.

Details: [`../platforms/polarfire/MEMORY.md`](../platforms/polarfire/MEMORY.md),
[`gdb-polarfire.md`](gdb-polarfire.md).

**Visualization:** LED `StateChanged` + UART hooks → JSON lines → agent →
WebSocket. Antmicro `serveVisualization` is a UX reference only (it needs an
HTTP server; we have `network=none`).

**GDB:** real `riscv64-unknown-elf-gdb` in the container (gdbgui’s MI model,
Wokwi’s one-click UX, not Wokwi’s WASM GDB).

---

## Examples

One platform contract, two languages.

| ID | Teaches |
|----|---------|
| `blinky` | GPIO, continue/step, LED freeze on halt |
| `uart-hello` | UART + breakpoints |
| `harts` | `info threads`, park/run U54s |
| `buggy-overflow` | Watchpoint / stack |

Rust (`riscv64gc-unknown-none-elf`, custom `_start` on hart 1) and Ada (GNAT /
Alire when available; **prebuilt ELF is fine**). Gallery shows language tabs.

---

## Non-goals (v1)

- Firecracker / Kata / dedicated servers
- Compiling user source on the server
- HSS / U-Boot / Linux boot
- FPGA fabric / Verilator
- Accounts, billing, teams
- Pixel-perfect Icicle silk-screen
- A second SoC before the PolarFire MVP definition of done is true

---

## Definition of done (MVP)

A stranger with a browser can:

1. Open the landing page, click Launch, pick PolarFire + Rust blinky.
2. See 4 LEDs, one blinking.
3. Pause / `Ctrl-C`; LEDs freeze; source line highlights.
4. `info threads` shows five harts; U54_1 is obvious.
5. `break` + `continue` works; UART example prints.
6. Upload a ~100 KB RV64 ELF of their own blinky and see LEDs move.
7. After 15 minutes or tab close, no leftover container or session dir.
8. A second user cannot talk to the first user’s socket or GDB.

Until those eight are true, do not add a second SoC.

---

## Open questions (defaults)

1. **Hostnames** — `hartlab.vesperforge.org` (site) + `play.vesperforge.org`
   (session host). Default: yes.
2. **Launch** — invite-only until ~20 clean external sessions. Default: yes.
3. **Auth** — anonymous + 1 session / IP for MVP. Login later.
4. **Cloud size** — 8 vCPU / 16 GB until RSS numbers say otherwise.
5. **Ada** — prebuilt ELF is acceptable if Alire RV64 is painful. Default: yes.
6. **Engine** — Docker Engine + `bollard`. Podman only if we want rootless.
7. **Brand** — product **HartLab**, operator Vesperforge.
