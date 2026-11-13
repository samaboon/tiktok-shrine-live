#!/usr/bin/env bash
# v0.2.4 shots: dance board/moves/hall/star + krathong board/normal/star
set -u
cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/workspace/tiktok-shrine-live-t49/target}"
if [ -z "${BIN:-}" ]; then
  if [ -x "${CARGO_TARGET_DIR}/release/tiktok-shrine-live" ]; then
    BIN=${CARGO_TARGET_DIR}/release/tiktok-shrine-live
  elif [ -x ./target/release/tiktok-shrine-live ]; then
    BIN=./target/release/tiktok-shrine-live
  else
    echo "missing release binary"; exit 1
  fi
fi
mkdir -p screenshots/dance/raw screenshots/krathong/raw screenshots/raw previews /tmp/v024shot
# GiftPicker cache_dir is the parent of --config
if [ -d /workspace/tiktok-shrine-live-gifts/gifts-cache ]; then
  ln -sfn /workspace/tiktok-shrine-live-gifts/gifts-cache /tmp/v024shot/gifts-cache
fi
if [ -z "${DISPLAY:-}" ]; then
  Xvfb :97 -screen 0 1920x1200x24 +extension GLX >/dev/null 2>&1 &
  XPID=$!; export DISPLAY=:97; sleep 1
fi

python3 - << 'PY'
import pathlib, re
t = pathlib.Path("config.toml").read_text()
t = re.sub(r'^check_on_start = .*', 'check_on_start = false', t, flags=re.M)
t = re.sub(r'^party_threshold = .*', 'party_threshold = 99', t, count=1, flags=re.M)
pathlib.Path("/tmp/v024shot/base.toml").write_text(t)
print("wrote /tmp/v024shot/base.toml")
PY
CFG=/tmp/v024shot/base.toml

shot() {
  local dir=$1 name=$2; shift 2
  timeout 150 "$BIN" --no-status --config "$CFG" --out "$dir/raw/$name.png" --preview --bg scene "$@" 2>&1 | tee /tmp/v024shot/${name}.log | grep -q saved || echo "FAILED $name"
  mv -f "$dir/raw/${name}_checker.png" "$dir/$name.png" 2>/dev/null || mv -f "$dir/raw/$name.png" "$dir/$name.png" 2>/dev/null || true
}

shot screenshots/dance board_idle --theme dance --shot-at 2.0 --seed 1
shot screenshots/dance board_live --theme dance --shot-at 5.0 --seed 21 \
  --demo "pray:1:มะลิ:Rose" --demo "pray:2:นก:Finger Heart" --demo "pray:3:สมชาย:Doughnut"
shot screenshots/dance move_rose --theme dance --shot-at 5.0 --seed 11 --demo "pray:1:มะลิ:Rose"
shot screenshots/dance move_doughnut --theme dance --shot-at 5.0 --seed 12 --demo "pray:3:สมชาย:Doughnut"
shot screenshots/dance hall_busy --theme dance --shot-at 6.5 --seed 42 \
  --demo "pray:1:มะลิ:Rose" --demo "pray:1:นก:TikTok" --demo "pray:2:ฝน:Finger Heart" \
  --demo "pray:1:ไผ่:Rose" --demo "pray:1:เจน:TikTok" --demo "pray:2:ต้น:Finger Heart" \
  --demo "pray:1:มิ้น:Rose" --demo "pray:1:แพรว:TikTok" --demo "pray:3:ดาว:Doughnut" \
  --demo "pray:3:โอม:Doughnut" --demo "pray:4:บีม:Hand Heart" --demo "pray:4:นุ่น:Hand Heart"
shot screenshots/dance hall_star --theme dance --shot-at 6.5 --seed 55 \
  --demo "pray:1:มะลิ:Rose" --demo "pray:1:นก:TikTok" --demo "pray:2:ฝน:Finger Heart" \
  --demo "pray:1:ไผ่:Rose" --demo "pray:3:ดาว:Doughnut" --demo "pray:4:บีม:Hand Heart" \
  --demo "pray:5:จักรพรรดิเพชร:Hand Heart"

shot screenshots/krathong board --theme krathong --shot-at 2.0 --seed 1
shot screenshots/krathong normal --theme krathong --shot-at 7.5 --seed 22 --demo "siamsi:3:สมชาย:Doughnut"
shot screenshots/krathong star --theme krathong --shot-at 7.5 --seed 55 --demo "siamsi:5:จักรพรรดิเพชร:Hand Heart"

# settings is a separate OS window; request_screenshot on that viewport does not finish in shot mode
if grep -q "^x = " /tmp/v024shot/base.toml; then
  sed -i "s/^x = .*/x = 1300/" /tmp/v024shot/base.toml
else
  sed -i "s/^\[window\]/[window]\nx = 1300\ny = 0/" /tmp/v024shot/base.toml
fi
"$BIN" --no-status --config "$CFG" --shot-at 99999 --settings --settings-tab 6 >/tmp/v024shot/settings_tab6_test.log 2>&1 &
SPID=$!
sleep 4
python3 - << "PY"
import os
from PIL import ImageGrab
img = ImageGrab.grab(xdisplay=os.environ["DISPLAY"])
img.crop((0, 0, 800, 760)).save("screenshots/settings_tab6_test.png")
print("saved screenshots/settings_tab6_test.png")
PY
kill $SPID 2>/dev/null
wait $SPID 2>/dev/null
# restore overlay position for remaining overlay shots? (none after this except gif — gif ignores x)

gif() {
  local name=$1 start=$2 end=$3 fps=$4; shift 4
  local dir
  dir=$(mktemp -d /tmp/v024gif_XXXX)
  timeout 200 "$BIN" --no-status --config "$CFG" --bg scene --record "$dir" "$start" "$end" "$fps" "$@" 2>&1 | tail -5
  local n
  n=$(ls -1 "$dir"/f*.png 2>/dev/null | wc -l)
  if [ "$n" -lt 3 ]; then
    echo "FAILED gif $name (frames=$n)"
    ls -la "$dir" || true
    rm -rf "$dir"
    return
  fi
  ffmpeg -y -loglevel error -framerate "$fps" -i "$dir/f%05d.png" \
    -vf "scale=540:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=96:stats_mode=single[p];[s1][p]paletteuse=dither=bayer" \
    -loop 0 "previews/${name}.gif"
  rm -rf "$dir"
  ls -lh "previews/${name}.gif"
}

gif dance_star 0 8 12 --theme dance --seed 55 \
  --demo "pray:1:มะลิ:Rose" --demo "pray:3:ดาว:Doughnut" --demo "pray:5:จักรพรรดิเพชร:Hand Heart"

[ -n "${XPID:-}" ] && kill $XPID
echo done
ls -lh screenshots/dance/board_idle.png screenshots/dance/board_live.png \
  screenshots/dance/move_rose.png screenshots/dance/move_doughnut.png \
  screenshots/dance/hall_busy.png screenshots/dance/hall_star.png \
  screenshots/krathong/board.png screenshots/krathong/normal.png screenshots/krathong/star.png \
  screenshots/settings_tab6_test.png previews/dance_star.gif 2>/dev/null
