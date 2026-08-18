# HartLab — roadmap

Source of truth for *what we build next*. Product intent lives in
[`VISION.md`](VISION.md).

Status key: `[x]` done in-tree, `[ ]` not done.

---

## Phase 0 — Fidelity spike (in progress)

**Goal:** PolarFire + GDB + LEDs work, and Renode survives the Docker flags.

- [x] Pin a workspace and PolarFire playground overlay (`platforms/polarfire/`).
- [x] Four LEDs on GPIO2 pins 16–19 in `playground.repl`.
- [x] Rust `blinky` ELF at `0x80000000`, U54_1 only (`./scripts/build-blinky.sh`).
- [x] e51-PC-copy + park U54_2–4 documented in `playground.resc` and
      `gdb-polarfire.md`.
- [x] Ada blinky source on the same pins (`examples/ada/blinky/`; prebuilt ELF
      still pending if Alire RV64 is unavailable).
- [x] IronPython hook writes JSON lines (`playground.py`).
- [x] Host ELF allowlist + GDB denylist (`crates/protocol`, tests passing).
- [x] Control / agent stubs (`apps/control`, `apps/agent`).
- [ ] Confirm Monitor `LED state changed` against a live Renode.
- [ ] Host GDB: `info threads` == 5, break on `rust_main`, LED freezes on halt.
- [ ] Same blinky inside Docker: `--network=none --read-only --cap-drop ALL
      --memory 1536m`. Record RSS, boot time, any missing capability.

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

**Goal:** same Docker runner, now on a Cloud VPS + Vercel.

- [ ] Provision ~8 vCPU / 16 GB. Docker, Caddy, `hartlab.service`.
- [ ] Image from GHCR, pin digest.
- [ ] `play.vesperforge.org` + Vercel landing (Launch Playground).
- [ ] Gallery + GDB cheatsheet + hart model + ELF rules.
- [ ] Hard cap 4–6 sessions; “lab is full”.
- [ ] Invite-only first week, then open.
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

1. ~~Repo skeleton + PolarFire pack + Rust blinky + fidelity script.~~ (landed)
2. ~~Ada blinky source.~~ (landed; prebuilt ELF still open)
3. Session Dockerfile + “blinky in a locked-down container”.
4. Agent protocol + fake backend tests.
5. Control session API + bollard + reaper.
6. Web playground talking to local Axum.
7. UART example + source viewer.
8. Hart example + hart selector.
9. Upload path + validator corpus.
10. Vercel landing + Caddy + Cloud runbook.
11. Exercises / buggy-overflow.
12. Viz polish + optional gVisor.

Until there is a production URL, land on `main`. If this becomes a Vesperforge
public site, switch to the Crownfall/Hearth rule: `development` first, then
`main`.

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

## Ops (when Phase 2 exists)

- systemd: `hartlab.service`, `caddy`, `docker`.
- Daily: `docker ps --filter label=app=hartlab` matches live sessions only.
- Cost: Vercel + one Cloud VPS (~€15–40/mo). No dedicated server.
- Capacity: raise the box or add a second VPS. Not k8s.
