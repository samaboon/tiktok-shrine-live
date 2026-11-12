#!/usr/bin/env bash
# Package the Windows build into dist/TikTokShrineLive-Windows-x64.zip
# Usage: packaging/make_windows_zip.sh path/to/tiktok-shrine-live.exe
set -euo pipefail
cd "$(dirname "$0")/.."
EXE=${1:-target/x86_64-pc-windows-gnu/release/tiktok-shrine-live.exe}
OUT=dist/TikTokShrineLive
rm -rf "$OUT" && mkdir -p "$OUT/assets/sprites"
cp "$EXE" "$OUT/TikTokShrineLive.exe"
cp config.example.toml "$OUT/config.toml"
cp assets/sprites/README.txt "$OUT/assets/sprites/" 2>/dev/null || true
cp docs/คู่มือ.md "$OUT/คู่มือ.md"
cp docs/ติดตั้ง-Windows.md "$OUT/ติดตั้ง-Windows.md"
cp assets/fonts/OFL.txt "$OUT/FONT-LICENSE-OFL.txt"
cp LICENSE "$OUT/LICENSE.txt"
cp NOTICE.md "$OUT/NOTICE.md"
# bundled Node.js runtime for the TikTok connector (no Node install needed); see packaging/fetch_node.sh
if [ -n "${RUNTIME_DIR:-}" ]; then
  mkdir -p "$OUT/runtime" && cp "$RUNTIME_DIR"/* "$OUT/runtime/"
  (cd dist && rm -rf rtpkg && mkdir -p rtpkg/runtime && cp "../$OUT/runtime/"* rtpkg/runtime/ && cd rtpkg && rm -f ../runtime-node-win-x64.zip && \
    if command -v zip >/dev/null; then zip -qr ../runtime-node-win-x64.zip runtime; \
    else python3 -c "import shutil; shutil.make_archive('../runtime-node-win-x64','zip','.','runtime')"; fi)
fi
printf 'ดับเบิลคลิก TikTokShrineLive.exe เพื่อเริ่ม\r\nไม่ต้องติดตั้ง Node.js (มากับโปรแกรมในโฟลเดอร์ runtime อย่าลบ)\r\nวิธีติดตั้ง/ไลฟ์ทีละขั้น: ติดตั้ง-Windows.md  การตั้งค่า: คู่มือ.md\r\n' > "$OUT/อ่านก่อน.txt"
(cd dist && rm -f TikTokShrineLive-Windows-x64.zip && \
  if command -v zip >/dev/null; then zip -qr TikTokShrineLive-Windows-x64.zip TikTokShrineLive; \
  else python3 -c "import shutil; shutil.make_archive('TikTokShrineLive-Windows-x64','zip','.','TikTokShrineLive')"; fi)
echo "built: dist/TikTokShrineLive-Windows-x64.zip"
