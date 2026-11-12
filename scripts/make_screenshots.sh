#!/usr/bin/env bash
# สร้างภาพตัวอย่างทั้งหมดแบบ headless (Linux + Xvfb) -> screenshots/
set -u
cd "$(dirname "$0")/.."
BIN=${BIN:-./target/release/tiktok-shrine-live}
OUT=screenshots
mkdir -p "$OUT/raw"
if [ -z "${DISPLAY:-}" ]; then
  Xvfb :93 -screen 0 1920x1200x24 +extension GLX >/dev/null 2>&1 &
  XPID=$!; export DISPLAY=:93; sleep 1
fi
N=0
shot() { # name, extra args...
  local name=$1; shift
  timeout 90 "$BIN" --no-status --out "$OUT/raw/$name.png" --preview "$@" 2>&1 | grep -q saved || echo "FAILED $name"
  mv -f "$OUT/raw/${name}_checker.png" "$OUT/$name.png" 2>/dev/null
}
run() { shot "$@" & N=$((N+1)); if [ $((N % 6)) -eq 0 ]; then wait; fi; }
NAMES=("สมชาย" "มะลิ" "เศรษฐีต้น" "เจ้าสัวเฮง" "จักรพรรดิเพชร")
for t in 1 2 3 4 5; do
  nm=${NAMES[$((t-1))]}
  run pray_t${t}_1_walk   --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 1.6
  run pray_t${t}_2_wai    --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 4.6
  run pray_t${t}_3_number --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 7.4
  run pray_t${t}_4_fireworks --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 8.4
  run siamsi_t${t}_1_shake   --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 4.6
  run siamsi_t${t}_2_stick   --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 6.9
  run siamsi_t${t}_3_fortune --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 9.6
done
run idle_transparent --shot-at 0.5
run mode_green_t5    --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 8.0 --bg green
run mode_scene_t4    --demo "siamsi:4:เจ้าสัวเฮง" --seed 52 --shot-at 9.6 --bg scene
run layout_full_t5   --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 8.0 --layout full
run layout_top_t5    --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 8.0 --layout top
wait
# settings window = separate OS window: grab it from the X screen (needs python3-pil)
sed 's/^x = .*/x = 1300/' config.toml > /tmp/_shotcfg.toml
setshot() { # name, args...
  local name=$1; shift
  "$BIN" --config /tmp/_shotcfg.toml --shot-at 99999 "$@" >/dev/null 2>&1 & local P=$!
  sleep 3
  python3 -c "from PIL import ImageGrab; ImageGrab.grab(xdisplay='$DISPLAY').crop((0,0,600,720)).save('$OUT/$name.png')"
  kill $P 2>/dev/null; wait $P 2>/dev/null
}
for i in 0 1 2 3 4 5 6; do setshot settings_tab$i --settings-tab $i; done
setshot settings_tab0_stopped --settings-tab 0 --stopped
[ -n "${XPID:-}" ] && kill $XPID
echo done
