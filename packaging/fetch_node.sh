#!/usr/bin/env bash
# Download the official portable Node.js runtime for the TikTok connector (bridge.cjs) so users
# never have to install Node.js. Verifies SHA-256 against nodejs.org SHASUMS256.txt.
# Usage: packaging/fetch_node.sh win|linux|mac OUTDIR
#   win   -> OUTDIR/node.exe
#   linux -> OUTDIR/node
#   mac   -> OUTDIR/node (universal arm64 + x86_64 via lipo, ad-hoc signed; run on macOS)
# plus OUTDIR/LICENSE-node.txt (Node.js is MIT; its bundled deps' notices are in that file)
# and OUTDIR/NODE_VERSION (read by the app's updater).
set -euo pipefail
NODE_VERSION=${NODE_VERSION:-$(cat "$(dirname "$0")/NODE_VERSION")}
PLAT=$1; OUT=$2
BASE="https://nodejs.org/dist/$NODE_VERSION"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
mkdir -p "$OUT"
curl -fsSL --retry 3 "$BASE/SHASUMS256.txt" -o "$TMP/SHASUMS256.txt"
get() { # get FILE -> $TMP/FILE (sha-checked)
  curl -fsSL --retry 3 "$BASE/$1" -o "$TMP/$1"
  want=$(grep " $1\$" "$TMP/SHASUMS256.txt" | cut -d' ' -f1)
  if command -v sha256sum >/dev/null; then have=$(sha256sum "$TMP/$1" | cut -d' ' -f1); else have=$(shasum -a 256 "$TMP/$1" | cut -d' ' -f1); fi
  [ -n "$want" ] && [ "$want" = "$have" ] || { echo "checksum mismatch for $1" >&2; exit 1; }
}
case "$PLAT" in
  win)
    f="node-$NODE_VERSION-win-x64"; get "$f.zip"
    (cd "$TMP" && unzip -q "$f.zip" "$f/node.exe" "$f/LICENSE")
    cp "$TMP/$f/node.exe" "$OUT/node.exe"; cp "$TMP/$f/LICENSE" "$OUT/LICENSE-node.txt" ;;
  linux)
    f="node-$NODE_VERSION-linux-x64"; get "$f.tar.xz"
    tar -xJf "$TMP/$f.tar.xz" -C "$TMP" "$f/bin/node" "$f/LICENSE"
    cp "$TMP/$f/bin/node" "$OUT/node"; cp "$TMP/$f/LICENSE" "$OUT/LICENSE-node.txt"; chmod 755 "$OUT/node" ;;
  mac)
    for a in arm64 x64; do
      f="node-$NODE_VERSION-darwin-$a"; get "$f.tar.gz"
      tar -xzf "$TMP/$f.tar.gz" -C "$TMP" "$f/bin/node" "$f/LICENSE"
    done
    lipo -create -output "$OUT/node" "$TMP/node-$NODE_VERSION-darwin-arm64/bin/node" "$TMP/node-$NODE_VERSION-darwin-x64/bin/node"
    chmod 755 "$OUT/node"
    # lipo invalidates the upstream signature -> ad-hoc sign the nested executable (no hardened runtime, so V8 JIT works)
    codesign --force -s - "$OUT/node"
    cp "$TMP/node-$NODE_VERSION-darwin-arm64/LICENSE" "$OUT/LICENSE-node.txt" ;;
  *) echo "unknown platform $PLAT" >&2; exit 2 ;;
esac
echo "$NODE_VERSION" > "$OUT/NODE_VERSION"
ls -la "$OUT"
