#!/usr/bin/env bash
# Load the PolarFire playground in Renode (headless) and print the GDB contract.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ELF="$ROOT/examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky"

if ! command -v renode >/dev/null 2>&1; then
  echo "renode not on PATH. Install the portable package, then re-run." >&2
  exit 1
fi

if [[ ! -f "$ELF" ]]; then
  echo "building blinky ELF..."
  cargo build --release --manifest-path "$ROOT/examples/rust/blinky/Cargo.toml"
fi

export HWSTATE_PATH="${HWSTATE_PATH:-/tmp/hartlab-hw.jsonl}"
rm -f "$HWSTATE_PATH"

echo "Starting Renode (batch). GDB stub will listen on :3333"
echo "In another terminal:"
echo "  riscv64-unknown-elf-gdb $ELF"
echo "  (gdb) target remote :3333"
echo "  (gdb) info threads"
echo "  (gdb) thread 2"
echo "  (gdb) break rust_main"
echo "  (gdb) continue"
echo

exec renode --disable-xwt --console \
  -e "\$ROOT=@$ROOT" \
  -e "\$bin=@$ELF" \
  -e "include @$ROOT/platforms/polarfire/playground.resc"
