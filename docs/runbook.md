# HartLab runbook (single operator)

## Dev

```bash
cargo test --workspace
./scripts/build-blinky.sh
./scripts/fetch-host-tools.sh    # Renode + GDB -> ~/.cache/hartlab/tools
./tests/fidelity/run-live.sh     # Phase 0 gate
# Docker lock-down (needs Docker Engine):
#   docker build -f infra/docker/fidelity.Dockerfile -t hartlab-fidelity:dev .
#   ./tests/fidelity/run-docker.sh
cargo run -p hartlab-control     # 127.0.0.1:8080 stub only
```

## Prod (U3 — not live yet)

- Site: https://hartlab.vesperforge.org (Vercel)
- Session host: `play.hartlab.vesperforge.org` (Caddy → Axum `:8080`)
- Hetzner Cloud, ~8 vCPU / 16 GB
- systemd: `hartlab.service` + `docker`
- `docker ps --filter label=app=hartlab` should match live sessions only

## Leftovers

If Axum dies, reap orphans:

```bash
docker ps -aq --filter label=app=hartlab | xargs -r docker rm -f
```
