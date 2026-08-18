#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec cargo build --release --target riscv64gc-unknown-none-elf \
  --manifest-path "$ROOT/examples/rust/blinky/Cargo.toml"
