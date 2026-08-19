# HartLab — roadmap

Source of truth for *what we build next*. Product intent lives in
[`VISION.md`](VISION.md). Do not add a second vision/plan file.

Status key: `[x]` done in-tree, `[ ]` not done.

---

## Usable public site — https://hartlab.vesperforge.org

This is the near-term program. It is a **cut** of Phases 0–2, not a new
product. Full 8-point MVP in the vision (UART gallery, ELF upload, Ada
binary) comes **after** this URL is real.

### What “usable” means

A stranger with an invite opens **https://hartlab.vesperforge.org**, clicks
Launch, and debugs the curated **Rust blinky** on PolarFire:

1. Landing + Launch (no account).
2. Four LEDs; LED1 blinks; graphic freezes on halt.
3. Toolbar: Continue, Step, Pause / Reset Halt.
4. Source line highlights on `rust_main` (DWARF is already in the teaching ELF).
5. Toggleable GDB terminal (`info threads` shows five harts; `thread 2` is U54_1).
6. 15 min TTL / tab close destroys the container (`docker ps` clean).
7. Visitor B cannot attach to visitor A’s session.

**Not in this cut:** Ada gallery ELF, UART example, custom ELF upload,
gVisor, accounts, a second SoC, pixel-perfect Icicle art.

### Hostnames (locked)

| Host | Where | Role |
|------|--------|------|
| `hartlab.vesperforge.org` | Vercel | Site + playground SPA |
| `play.hartlab.vesperforge.org` | Hetzner Cloud + Caddy | Session API + WebSocket only |

The browser never talks to Renode or GDB. Both names are Vesperforge DNS.

### Gap from this tree

| Have | Missing for the URL |
|------|---------------------|
| PolarFire overlay, teaching blinky, ELF allowlist, GDB denylist | Session image that **runs** Renode + GDB + agent |
| Live host spike on a Mac ([PR #1](https://github.com/angdav2112/hartlab/pull/1)) | **`cap-drop ALL` proven** (CI or a Linux box with Docker) |
| Control/agent **stubs** (no container, no `/ws`) | Real `POST /v1/sessions`, WS mux, reaper, `bollard` |
| `apps/web` placeholder | Landing + playground SPA |
| Draft Caddy/systemd | Cloud VPS, DNS, Vercel project, invite list |

Do **not** point the domain at a stub. Isolation gate first.

### Sequence (one operator)

**U0 — Isolation gate (days)**  
Bake Renode + GDB + agent into `infra/docker/fidelity.Dockerfile` /
`session.Dockerfile`. `./tests/fidelity/run-docker.sh` green (GitHub
Actions on `ubuntu-latest`, or any Linux Docker host). Record RSS. If
Renode needs a capability, grant that one and write it down. If DDR
mapping is huge, shrink the playground `.repl` (keep `0x80000000`).

**U1 — Real session, localhost only (1–2 weeks)**  
Agent starts Renode + GDB MI, `monitor start`, streams LED JSON. Control
creates one container, proxies one WSS (`gdb` + `hw`), reaps on TTL.
Integration test: `ready` → continue → LED `true` → halt → freeze →
`docker rm`. No public bind (`127.0.0.1`).

**U2 — Browser (1–2 weeks, after U1 talks JSON)**  
`apps/web` on Vercel: landing (“Launch Playground”), SVG board, toolbar,
xterm.js, source pane. Talks only to `play.hartlab.vesperforge.org`.
Preview URL is enough until DNS.

**U3 — Domain (few days, overlaps U2)**  
Hetzner Cloud ~8 vCPU / 16 GB. Docker Engine, Caddy, `hartlab.service`.
DNS: `hartlab` → Vercel, `play.hartlab` → VPS. Pin session image digest.
`max_concurrent = 4`, 1 session / IP, invite-only (shared link or IP
allowlist). Runbook: `docker ps --filter label=app=hartlab`.

**Stop.** Invite ~20 external sessions. Then UART, upload, Ada ELF, polish
(Phases 3–4). Not before.

---

## Phase 0 — Fidelity spike (in progress)

**Goal:** PolarFire + GDB + LEDs work, and Renode survives the Docker flags.
Control/agent crates in this repo are **placeholders**, not spike progress
(see [`spikes/2026-08-18-cursor-review.md`](spikes/2026-08-18-cursor-review.md)).

- [x] Pin a workspace and PolarFire playground overlay (`platforms/polarfire/`).
- [x] Four LEDs on GPIO2 pins 16–19 in `playground.repl`.
- [x] Rust `blinky` ELF at `0x80000000`, U54_1 only (`./scripts/build-blinky.sh`).
- [x] Blinky is a teaching ELF: `ENTRY(_start)`, `debug = 2`, no LTO.
- [x] e51-PC-copy + park U54_2–4 documented in `playground.resc` and
      `gdb-polarfire.md`.
- [ ] Ada gallery ELF — source exhibit only; needs a prebuilt RV64 binary.
- [x] IronPython hook writes JSON lines and **fails loudly** if LEDs do not bind.
- [x] Host ELF allowlist (PT_LOAD at `0x80000000`) + default-deny GDB filter.
- [ ] Confirm `StateChanged` / LED JSON against a **live** Renode.
- [ ] Host GDB: `info threads` == 5, break on `rust_main`, LED freezes on halt.
- [ ] Same blinky inside Docker: `--network=none --read-only --cap-drop ALL
      --memory 1536m` **with Renode+GDB actually in the image**. Record RSS.
- [ ] DDR RSS spike — shrink playground overlay if 2 GB mapping is real.

**Exit:** a 2-minute recording for the landing page, plus a note “Renode works
with cap-drop ALL: yes/no”. If multi-hart GDB or GPIO2 LEDs fail, **stop**
before growing Axum.

---

## Phase 1 — Session MVP on Docker (~2–3 weeks)

**Goal:** browser → Axum → session container → Renode/GDB/agent. This *is* the
production isolation model, just on localhost.

- [ ] Agent protocol against a fake Renode + more tests.
- [ ] `hartlab-control`: `POST /v1/sessions`, WS mux, reaper, `bollard` runner.
- [ ] Session Dockerfile as in [`VISION.md`](VISION.md) (Renode + GDB baked in).
- [ ] `apps/web` playground: SVG LEDs, toolbar, xterm, source pane, UART.
- [ ] Curated blinky (Rust + Ada ELF) + upload path.
- [ ] Integration test: `ready` → continue → LED `true` → halt → freeze.

---

## Phase 2 — Public site on Hetzner Cloud (~2 weeks)

**Goal:** U3 above. Same Docker runner, on Cloud + Vercel.

- [ ] Provision ~8 vCPU / 16 GB. Docker, Caddy, `hartlab.service`.
- [ ] Image from GHCR, pin digest.
- [ ] `hartlab.vesperforge.org` (Vercel) + `play.hartlab.vesperforge.org` (VPS).
- [ ] Launch page + blinky only (full gallery after the usable cut).
- [ ] Hard cap 4 sessions; “lab is full”.
- [ ] Invite-only until ~20 clean external sessions.
- [ ] Reaper + disk checks in the runbook.

---

## Phase 3 — Visualization, teaching, optional gVisor (~2–3 weeks)

- [ ] Better board art; clickable SW2 (GPIO2 pin 30).
- [ ] Register groups + memory hexdump closer to SoftConsole.
- [ ] In-UI exercises (`buggy-overflow`).
- [ ] Disassembly-only path for stripped uploads.
- [ ] Spike gVisor; enable if Renode stays stable.
- [ ] Optional PWM/gauge for curated examples.

---

## Phase 4 — Multi-target + polish

- [ ] Second target only after PolarFire is boring. Best second: Cortex-M
      (STM32 blinky) or single-hart RISC-V — not another 5-hart Linux SoC.
- [ ] Metrics: start time, OOM, Renode crashes, unique ELF hashes.
- [ ] Private-instance mode: source upload + in-container compile (Rust first),
      still `network=none`.
- [ ] Scale = a second Cloud VPS, not k8s.

---

## Research / spikes (notes go under `docs/spikes/`)

1. LED hook — `StateChanged` vs GPIO2 write hook if the LED model does not fire.
2. Multi-hart GDB script — 5 threads, break on U54_1, `info registers`.
3. DDR size / RSS — stock 2 GB vs 64 MB playground `.repl`. Keep `0x80000000`.
4. Docker hardening — exact `docker run` flags from the vision. Record caps.
5. gVisor — 4-hour timebox. Backlog if it breaks Renode/.NET.
6. GDB/MI × xterm — toolbar + typed CLI; denylist rejects `shell`.
7. ELF corpus — RV64 static accept; x86, Linux dyn, truncated, 5 MiB reject.
8. Ada RV64 — 4-hour timebox; otherwise CI-prebuilt ELF only.

---

## Implementation order (PRs)

Toward **hartlab.vesperforge.org** (usable cut). Feature PRs → `development`.

1. ~~Repo skeleton + PolarFire pack + Rust blinky.~~ (landed)
2. ~~Host fidelity notes + protocol hardening.~~ (landed; live Renode on PR #1)
3. **U0** — Session/fidelity image with Renode+GDB; `run-docker.sh` green.
4. **U1** — Agent starts Renode/GDB; control + bollard + reaper + WS.
5. **U2** — `apps/web` landing + blinky playground (Vercel preview).
6. **U3** — DNS + Caddy + VPS; invite-only `hartlab.vesperforge.org`.
7. UART example + source/UART pane (full MVP #5).
8. ELF upload + validator corpus (full MVP #6).
9. Ada prebuilt gallery ELF.
10. Hart overview UI (terminal already has `info threads`).
11. Exercises / buggy-overflow / viz polish.

Branch flow: [`CONTRIBUTING.md`](../CONTRIBUTING.md). Feature → `development`
→ smoke → `main`. Never open a feature PR against `main`.

---

## Risks

| Risk | Mitigation |
|------|------------|
| Renode/.NET/GDB ELF parser RCE | Assumed possible. `network=none`, cap-drop, ro root, allowlist, invite-only until the reaper is trusted. |
| GPIO2 LED mapping wrong | Phase 0 live Renode. Fallback: sysbus write hook. |
| GDB `load` only starts e51 | Reset macro + UI “thread 2 (U54_1)”. |
| 2 GB DDR RSS | Measure; shrink playground DDR, keep the address. |
| Forbidden capability needed | Spike the Docker flags. Grant the minimum. |
| Docker API as privilege | Axum only client. No socket in sessions. |
| Users expect in-browser compile | Gallery + local toolchain recipe. ELF upload only. |
| xterm becomes a host shell | No `/bin/sh` PTY. GDB only. Denylist. |
| Monitor / viz HTTP leaked | Never start them. |
| Leftover containers | Reaper + `app=hartlab` labels. |
| Ada runtime friction | Prebuilt ELF is OK. |
| Scope creep | No second SoC until MVP DoD. |

---

## Ops (when U3 exists)

- systemd: `hartlab.service`, `caddy`, `docker`.
- Daily: `docker ps --filter label=app=hartlab` matches live sessions only.
- Cost: Vercel + one Cloud VPS (~€15–40/mo). No dedicated server.
- Capacity: raise the box or add a second VPS. Not k8s.
- Site: https://hartlab.vesperforge.org — session host:
  `play.hartlab.vesperforge.org` (not published as a product URL).
