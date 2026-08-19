# HartLab runbook (single operator)

## Dev

```bash
cargo test --workspace
./scripts/build-blinky.sh
./tests/fidelity/run-blinky.sh   # needs Renode on PATH
cargo run -p hartlab-control     # :8080
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
