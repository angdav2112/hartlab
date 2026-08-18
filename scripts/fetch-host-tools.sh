#!/usr/bin/env bash
# Download pinned Renode + RISC-V GDB into $HARTLAB_TOOLS.
# Safe to re-run. Does not write into the git work tree.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/tools/versions.env"

DEST="${HARTLAB_TOOLS:-$HOME/.cache/hartlab/tools}"
DL="$DEST/downloads"
mkdir -p "$DL"

uname_s="$(uname -s)"
uname_m="$(uname -m)"

renode_asset=""
xpack_asset=""
case "$uname_s-$uname_m" in
  Darwin-arm64)
    renode_asset="$RENODE_ASSET_DARWIN_ARM64"
    xpack_asset="$XPACK_ASSET_DARWIN_ARM64"
    ;;
  Linux-x86_64)
    renode_asset="$RENODE_ASSET_LINUX_X64"
    xpack_asset="$XPACK_ASSET_LINUX_X64"
    ;;
  Linux-aarch64)
    renode_asset="$RENODE_ASSET_LINUX_ARM64"
    xpack_asset="$XPACK_ASSET_LINUX_ARM64"
    ;;
  *)
    echo "unsupported host: $uname_s $uname_m" >&2
    exit 1
    ;;
esac

fetch() {
  local url="$1" out="$2"
  if [[ -f "$out" ]]; then
    echo "have $(basename "$out")"
    return
  fi
  echo "fetch $url"
  curl -fL --retry 3 -o "$out.partial" "$url"
  mv "$out.partial" "$out"
}

fetch "https://github.com/renode/renode/releases/download/${RENODE_TAG}/${renode_asset}" \
  "$DL/$renode_asset"
fetch "https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/${XPACK_TAG}/${xpack_asset}" \
  "$DL/$xpack_asset"

# --- xPack ---
xpack_dir="$DEST/xpack-riscv-none-elf-gcc-${XPACK_VERSION}"
if [[ ! -x "$xpack_dir/bin/riscv-none-elf-gdb" ]]; then
  echo "extract $xpack_asset"
  tar -xzf "$DL/$xpack_asset" -C "$DEST"
fi

# --- Renode ---
renode_dir="$DEST/renode-${RENODE_VERSION}"
mkdir -p "$renode_dir"
if [[ "$renode_asset" == *.dmg ]]; then
  if [[ ! -e "$renode_dir/.extracted" ]]; then
    echo "attach $renode_asset"
    mnt="$(hdiutil attach -nobrowse -readonly "$DL/$renode_asset" | awk '/\/Volumes\//{print $NF; exit}')"
    echo "mounted $mnt"
    # Portable .NET DMG is a folder of binaries, not necessarily an .app
    ditto "$mnt" "$renode_dir"
    hdiutil detach "$mnt" >/dev/null
    touch "$renode_dir/.extracted"
  fi
else
  if [[ ! -e "$renode_dir/.extracted" ]]; then
    echo "extract $renode_asset"
    tar -xzf "$DL/$renode_asset" -C "$renode_dir" --strip-components=1
    touch "$renode_dir/.extracted"
  fi
fi

# Resolve launcher (Linux portable is ./renode; macOS DMG is Renode.app)
renode_bin=""
for c in \
  "$renode_dir/renode" \
  "$renode_dir/macos/renode" \
  "$renode_dir/bin/renode" \
  "$renode_dir/Renode.app/Contents/MacOS/renode"
do
  if [[ -x "$c" ]]; then
    renode_bin="$c"
    break
  fi
done
if [[ -z "$renode_bin" ]]; then
  echo "could not find renode launcher under $renode_dir:" >&2
  find "$renode_dir" -maxdepth 4 \( -name 'renode' -o -name 'Renode' \) | head >&2
  exit 1
fi

gdb_bin="$xpack_dir/bin/riscv-none-elf-gdb"
[[ -x "$gdb_bin" ]] || { echo "missing $gdb_bin" >&2; exit 1; }

cat > "$DEST/env.sh" <<EOF
export HARTLAB_TOOLS="$DEST"
export RENODE="$renode_bin"
export RISCV_GDB="$gdb_bin"
export PATH="$xpack_dir/bin:\$PATH"
EOF

echo "wrote $DEST/env.sh"
echo "  RENODE=$renode_bin"
echo "  RISCV_GDB=$gdb_bin"
