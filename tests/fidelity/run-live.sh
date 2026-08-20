#!/usr/bin/env bash
# Phase 0 host spike. Requires tools from scripts/fetch-host-tools.sh.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TOOLS="${HARTLAB_TOOLS:-$HOME/.cache/hartlab/tools}"

if [[ ! -f "$TOOLS/env.sh" ]]; then
  echo "fetching host tools into $TOOLS"
  "$ROOT/scripts/fetch-host-tools.sh"
fi
# shellcheck disable=SC1091
source "$TOOLS/env.sh"

if [[ ! -f "$ROOT/examples/rust/blinky/target/riscv64gc-unknown-none-elf/release/blinky" ]]; then
  "$ROOT/scripts/build-blinky.sh"
fi

export HARTLAB_FIDELITY_DIR="${HARTLAB_FIDELITY_DIR:-/tmp/hartlab-fidelity}"
export HWSTATE_PATH="${HWSTATE_PATH:-$HARTLAB_FIDELITY_DIR/hw.jsonl}"
mkdir -p "$HARTLAB_FIDELITY_DIR"

exec python3 "$ROOT/tests/fidelity/run_live.py"
