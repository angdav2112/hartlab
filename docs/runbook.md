# HartLab runbook (single operator)

## Dev

```bash
cargo test --workspace
./scripts/build-blinky.sh
./tests/fidelity/run-blinky.sh   # needs Renode on PATH
cargo run -p hartlab-control     # :8080
```

## Prod (later)

- Hetzner Cloud, 8 vCPU / 16 GB class
- Caddy for TLS on `play.<domain>`
- systemd: `hartlab.service` + `docker`
- `docker ps --filter label=app=hartlab` should match live sessions only

## Leftovers

If Axum dies, reap orphans:

```bash
docker ps -aq --filter label=app=hartlab | xargs -r docker rm -f
```
