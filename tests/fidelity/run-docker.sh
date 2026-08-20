#!/usr/bin/env bash
# Run the live spike inside the lock-down flags from docs/VISION.md.
# Requires Docker and a pre-built hartlab-fidelity image (see fidelity.Dockerfile).
set -euo pipefail

IMAGE="${1:-hartlab-fidelity:dev}"

echo "=== docker run lock-down: $IMAGE ==="
# /tmp stays noexec (JSONL + the copied work tree). .NET/Renode JIT and
# mmap need an exec tmp; that is /var/tmp. HOME must be writable (config).
# .NET host itself was extracted at image-build into /opt/dotnet-extract.
docker run --rm \
  --name "hartlab-fidelity-$$" \
  --network=none \
  --read-only \
  --cap-drop ALL \
  --security-opt no-new-privileges \
  --memory 1536m \
  --memory-swap 1536m \
  --cpus 1 \
  --pids-limit 256 \
  --tmpfs /tmp:rw,noexec,nosuid,size=128m,uid=10000,gid=10000 \
  --tmpfs /var/tmp:rw,nosuid,size=128m,uid=10000,gid=10000 \
  --tmpfs /home/session:rw,nosuid,size=32m,uid=10000,gid=10000 \
  --user 10000:10000 \
  -e DOTNET_BUNDLE_EXTRACT_BASE_DIR=/opt/dotnet-extract \
  -e TMPDIR=/var/tmp \
  -e TMP=/var/tmp \
  "$IMAGE"

echo "=== cap-drop ALL: yes (container exited 0) ==="
