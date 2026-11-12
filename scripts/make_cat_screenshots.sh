#!/usr/bin/env bash
# ภาพตัวอย่างโหมดให้อาหารแมว (headless Linux + Xvfb) -> screenshots/cat/ และ previews/cat_*.gif
# เวลาในเกม: เดินเข้าสูงสุด 2.4 วิ | เทอาหาร 1.15 วิ | กิน 2.25 วิ | การ์ดขอบคุณ 5+0.5*ระดับ วิ | เดินออก 1.65 วิ
set -u
cd "$(dirname "$0")/.."
if [ -z "${BIN:-}" ]; then
  if [ -x ./.bin/tiktok-shrine-live ]; then
    BIN=./.bin/tiktok-shrine-live
  elif [ -x "${CARGO_TARGET_DIR:-}/release/tiktok-shrine-live" ]; then
    BIN=${CARGO_TARGET_DIR}/release/tiktok-shrine-live
  elif [ -x ./target/release/tiktok-shrine-live ]; then
    BIN=./target/release/tiktok-shrine-live
  else
    BIN=${CARGO_TARGET_DIR:-./target}/release/tiktok-shrine-live
  fi
fi
OUT=screenshots/cat
mkdir -p "$OUT/raw" previews
if [ -z "${DISPLAY:-}" ]; then
  Xvfb :94 -screen 0 1920x1200x24 +extension GLX >/dev/null 2>&1 &
  XPID=$!; export DISPLAY=:94; sleep 1
fi
N=0
shot() {
  local name=$1; shift
  timeout 120 "$BIN" --no-status --out "$OUT/raw/$name.png" --preview "$@" 2>&1 | grep -q saved || echo "FAILED $name"
  mv -f "$OUT/raw/${name}_checker.png" "$OUT/$name.png" 2>/dev/null
}
run() { shot "$@" & N=$((N+1)); if [ $((N % 6)) -eq 0 ]; then wait; fi; }

# idle still
run idle_still            --theme cat --shot-at 3
run idle_still_scene      --theme cat --bg scene --shot-at 3

# tier 1 มะลิ
run t1_1_walk   --theme cat --demo "pray:1:มะลิ" --seed 11 --shot-at 1.3
run t1_2_pour   --theme cat --demo "pray:1:มะลิ" --seed 11 --shot-at 3.1
run t1_3_eat    --theme cat --demo "pray:1:มะลิ" --seed 11 --shot-at 5.0
run t1_4_card   --theme cat --demo "pray:1:มะลิ" --seed 11 --shot-at 7.4
run t1_eat_scene --theme cat --bg scene --demo "pray:1:มะลิ" --seed 11 --shot-at 5.0

# tier 5 จักรพรรดิเพชร
run t5_1_walk      --theme cat --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 1.3
run t5_2_pour      --theme cat --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 3.2
run t5_3_eat       --theme cat --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 5.1
run t5_4_card      --theme cat --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 7.5
run t5_eat_scene   --theme cat --bg scene --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 5.1
run t5_card_scene  --theme cat --bg scene --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 7.5
run t5_thanks      --theme cat --bg scene --demo "siamsi:5:จักรพรรดิเพชร" --seed 57 --shot-at 7.5

# 3 senders in a visible queue
run queue_3        --theme cat --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 3.0
run queue_3_eat    --theme cat --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 5.1
run queue_3_scene  --theme cat --bg scene --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 3.0
run queue_3_eat_scene --theme cat --bg scene --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 5.1

wait

# settings (optional; needs python3-pil)
if python3 -c "from PIL import ImageGrab" 2>/dev/null; then
  sed -e 's/^x = .*/x = 1300/' -e 's/^check_on_start = .*/check_on_start = false/' config.toml > /tmp/_catshotcfg.toml
  "$BIN" --config /tmp/_catshotcfg.toml --shot-at 99999 --settings-tab 0 --theme cat >/dev/null 2>&1 & localP=$!
  sleep 3
  python3 -c "from PIL import ImageGrab; ImageGrab.grab(xdisplay='$DISPLAY').crop((0,0,600,720)).save('$OUT/settings_tab0.png')" || true
  kill $localP 2>/dev/null; wait $localP 2>/dev/null
fi

# preview GIFs from --record frames (scene bg, compact palette)
gif() {
  local name=$1; shift
  local dir
  dir=$(mktemp -d /tmp/catgif_XXXX)
  timeout 180 "$BIN" --no-status --theme cat --bg scene --record "$dir" "$@" 2>&1 | tail -3
  local n
  n=$(ls -1 "$dir"/f*.png 2>/dev/null | wc -l)
  if [ "$n" -lt 3 ]; then
    echo "FAILED gif $name (frames=$n)"
    rm -rf "$dir"
    return
  fi
  ffmpeg -y -loglevel error -framerate 12 -i "$dir/f%05d.png" \
    -vf "scale=540:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=96:stats_mode=single[p];[s1][p]paletteuse=dither=bayer" \
    -loop 0 "previews/${name}.gif"
  rm -rf "$dir"
  ls -lh "previews/${name}.gif"
}

gif cat_t1 0 8 12 --demo "pray:1:มะลิ" --seed 11
gif cat_t5 0 9 12 --demo "pray:5:จักรพรรดิเพชร" --seed 55
gif cat_queue 0 8 12 --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42

[ -n "${XPID:-}" ] && kill $XPID
echo done
ls -lh "$OUT"/*.png previews/cat_*.gif 2>/dev/null
