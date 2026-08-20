#!/usr/bin/env bash
# Interactive Renode console for the playground. For the automated gate use
# ./tests/fidelity/run-live.sh instead.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ELF="$ROOT/examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky"
TOOLS="${HARTLAB_TOOLS:-$HOME/.cache/hartlab/tools}"
if [[ -f "$TOOLS/env.sh" ]]; then
  # shellcheck disable=SC1091
  source "$TOOLS/env.sh"
fi

if ! command -v "${RENODE:-renode}" >/dev/null 2>&1 && [[ ! -x "${RENODE:-}" ]]; then
  echo "renode not on PATH. Run ./scripts/fetch-host-tools.sh" >&2
  exit 1
fi
renode_bin="${RENODE:-renode}"

if [[ ! -f "$ELF" ]]; then
  echo "building blinky ELF..."
  "$ROOT/scripts/build-blinky.sh"
fi

export HWSTATE_PATH="${HWSTATE_PATH:-/tmp/hartlab-hw.jsonl}"
rm -f "$HWSTATE_PATH"

echo "Starting Renode. GDB stub will listen on :3333"
echo "In another terminal:"
echo "  ${RISCV_GDB:-riscv-none-elf-gdb} $ELF"
echo "  (gdb) target remote :3333"
echo "  (gdb) info threads"
echo "  (gdb) thread 2"
echo "  (gdb) break rust_main"
echo "  (gdb) monitor start"
echo "  (gdb) continue"
echo

cd "$ROOT"
exec "$renode_bin" --disable-xwt --plain \
  -e "include @platforms/polarfire/playground.resc"
