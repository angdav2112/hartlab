#!/usr/bin/env bash
# Run the live spike inside the lock-down flags from docs/VISION.md.
# Requires Docker and a pre-built hartlab-fidelity image (see fidelity.Dockerfile).
set -euo pipefail

IMAGE="${1:-hartlab-fidelity:dev}"

echo "=== docker run lock-down: $IMAGE ==="
# /tmp must be writable (JSONL + logs). noexec is fine: tools are in /opt.
docker run --rm \
  --name "hartlab-fidelity-$$" \
  --network=none \
  --read-only \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  --memory 1536m \
  --memory-swap 1536m \
  --cpus 1 \
  --pids-limit 64 \
  --tmpfs /tmp:rw,noexec,nosuid,size=32m \
  --tmpfs /home/session:rw,noexec,nosuid,size=8m \
  --user 10000:10000 \
  "$IMAGE"

echo "=== cap-drop ALL: yes (container exited 0) ==="
