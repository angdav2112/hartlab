#!/usr/bin/env bash
# Run the live spike inside the lock-down flags from docs/VISION.md.
# Requires Docker and a pre-built hartlab-fidelity image (see fidelity.Dockerfile).
set -euo pipefail

IMAGE="${1:-hartlab-fidelity:dev}"

echo "=== docker run lock-down: $IMAGE ==="
# /tmp must be writable (JSONL + logs). noexec is fine: tools are in /opt.
# tmpfs is root:755 unless uid is set — uid 10000 must own the mounts.
# /tmp stays noexec (logs/JSONL). Renode's portable .NET bundle must extract
# to an exec tmpfs or it dies with DOTNET_BUNDLE_EXTRACT_BASE_DIR.
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
  --tmpfs /tmp:rw,noexec,nosuid,size=64m,uid=10000,gid=10000 \
  --tmpfs /dotnet-extract:rw,nosuid,size=128m,uid=10000,gid=10000 \
  --tmpfs /home/session:rw,noexec,nosuid,size=8m,uid=10000,gid=10000 \
  --user 10000:10000 \
  -e DOTNET_BUNDLE_EXTRACT_BASE_DIR=/dotnet-extract \
  "$IMAGE"

echo "=== cap-drop ALL: yes (container exited 0) ==="
