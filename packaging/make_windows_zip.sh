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
cp assets/fonts/OFL.txt "$OUT/FONT-LICENSE-OFL.txt"
printf 'ดับเบิลคลิก TikTokShrineLive.exe เพื่อเริ่ม\r\nอ่านวิธีใช้ทั้งหมดใน คู่มือ.md\r\n' > "$OUT/อ่านก่อน.txt"
(cd dist && rm -f TikTokShrineLive-Windows-x64.zip && \
  if command -v zip >/dev/null; then zip -qr TikTokShrineLive-Windows-x64.zip TikTokShrineLive; \
  else python3 -c "import shutil; shutil.make_archive('TikTokShrineLive-Windows-x64','zip','.','TikTokShrineLive')"; fi)
echo "built: dist/TikTokShrineLive-Windows-x64.zip"
