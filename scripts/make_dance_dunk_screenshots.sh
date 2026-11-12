#!/usr/bin/env bash
# ภาพตัวอย่างโหมดเต้น (รวมทุกเวที) และโหมดปาตกน้ำ -> screenshots/dance/ screenshots/dunk/ previews/
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
mkdir -p screenshots/dance/raw screenshots/dunk/raw previews /tmp/ddshot
if [ -z "${DISPLAY:-}" ]; then
  Xvfb :96 -screen 0 1920x1200x24 +extension GLX >/dev/null 2>&1 &
  XPID=$!; export DISPLAY=:96; sleep 1
fi

write_cfg() {
  local venue=$1
  local out=/tmp/ddshot/dance_${venue}.toml
  python3 - "$venue" "$out" << 'PY'
import sys, pathlib, re
venue, out = sys.argv[1], sys.argv[2]
t = pathlib.Path("config.toml").read_text()
t = re.sub(r'^check_on_start = .*', 'check_on_start = false', t, flags=re.M)
t = re.sub(r'^venue = .*', f'venue = "{venue}"', t, flags=re.M)
pathlib.Path(out).write_text(t)
PY
  echo "$out"
}

N=0
PIDS=
shot() {
  local dir=$1 name=$2; shift 2
  timeout 120 "$BIN" --no-status --out "$dir/raw/$name.png" --preview "$@" 2>&1 | grep -q saved || echo "FAILED $name"
  mv -f "$dir/raw/${name}_checker.png" "$dir/$name.png" 2>/dev/null || true
}
run() { shot "$@" & PIDS="$PIDS $!"; N=$((N+1)); if [ $((N % 5)) -eq 0 ]; then wait $PIDS; PIDS=; fi; }
wait_shots() { if [ -n "${PIDS:-}" ]; then wait $PIDS; PIDS=; fi; }

declare -A VTIER
VTIER[floor]=1; VTIER[temple_fair]=1
VTIER[table]=2; VTIER[speaker]=2
VTIER[stage]=3; VTIER[parade]=3
VTIER[rooftop]=4; VTIER[foam]=4
VTIER[cloud]=5
for v in floor temple_fair table speaker stage parade rooftop foam cloud; do
  cfg=$(write_cfg "$v")
  t=${VTIER[$v]}
  run screenshots/dance "venue_$v" --config "$cfg" --theme dance --bg scene --demo "pray:${t}:มะลิ" --demo "pray:2:สมชาย" --demo "siamsi:1:นก" --seed 21 --shot-at 3.7
done

cfg_floor=$(write_cfg floor)
cfg_stage=$(write_cfg stage)
run screenshots/dance t1_dance        --config "$cfg_floor" --theme dance --demo "pray:1:มะลิ" --seed 11 --shot-at 3.7
run screenshots/dance t1_dance_scene  --config "$cfg_floor" --theme dance --bg scene --demo "pray:1:มะลิ" --seed 11 --shot-at 3.7
run screenshots/dance t3_dance        --config "$cfg_stage" --theme dance --demo "pray:3:สมชาย" --seed 33 --shot-at 3.7
run screenshots/dance t3_dance_scene  --config "$cfg_stage" --theme dance --bg scene --demo "pray:3:สมชาย" --seed 33 --shot-at 3.7
run screenshots/dance t5_dance        --config "$cfg_stage" --theme dance --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 4.2
run screenshots/dance t5_dance_scene  --config "$cfg_stage" --theme dance --bg scene --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 4.2
run screenshots/dance queue_3         --config "$cfg_floor" --theme dance --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 3.7
run screenshots/dance queue_3_scene   --config "$cfg_floor" --theme dance --bg scene --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42 --shot-at 3.7
run screenshots/dance idle_still_scene --config "$cfg_floor" --theme dance --bg scene --shot-at 3

python3 - << 'PY'
import pathlib, re
t = pathlib.Path("config.toml").read_text()
t = re.sub(r'^check_on_start = .*', 'check_on_start = false', t, flags=re.M)
pathlib.Path("/tmp/ddshot/dunk.toml").write_text(t)
PY
DCFG=/tmp/ddshot/dunk.toml
run screenshots/dunk t1_throw        --config "$DCFG" --theme dunk --demo "pray:1:มะลิ" --seed 11 --shot-at 2.1
run screenshots/dunk t1_throw_scene  --config "$DCFG" --theme dunk --bg scene --demo "pray:1:มะลิ" --seed 11 --shot-at 2.1
run screenshots/dunk t3_throw        --config "$DCFG" --theme dunk --demo "pray:3:สมชาย" --seed 33 --shot-at 3.0
run screenshots/dunk t3_throw_scene  --config "$DCFG" --theme dunk --bg scene --demo "pray:3:สมชาย" --seed 33 --shot-at 3.0
run screenshots/dunk t5_splash       --config "$DCFG" --theme dunk --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 5.2
run screenshots/dunk t5_splash_scene --config "$DCFG" --theme dunk --bg scene --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 5.2
run screenshots/dunk queue_3         --config "$DCFG" --theme dunk --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:2:นก" --seed 42 --shot-at 2.2
run screenshots/dunk queue_3_scene   --config "$DCFG" --theme dunk --bg scene --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:2:นก" --seed 42 --shot-at 2.2

wait_shots

gif() {
  local name=$1 start=$2 end=$3 fps=$4; shift 4
  local dir
  dir=$(mktemp -d /tmp/ddgif_XXXX)
  timeout 200 "$BIN" --no-status --bg scene --record "$dir" "$start" "$end" "$fps" "$@" 2>&1 | tail -3
  local n
  n=$(ls -1 "$dir"/f*.png 2>/dev/null | wc -l)
  if [ "$n" -lt 3 ]; then
    echo "FAILED gif $name (frames=$n)"
    rm -rf "$dir"
    return
  fi
  ffmpeg -y -loglevel error -framerate "$fps" -i "$dir/f%05d.png" \
    -vf "scale=540:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=96:stats_mode=single[p];[s1][p]paletteuse=dither=bayer" \
    -loop 0 "previews/${name}.gif"
  rm -rf "$dir"
  ls -lh "previews/${name}.gif"
}

gif dance_t1 0 9 12 --config "$cfg_floor" --theme dance --demo "pray:1:มะลิ" --seed 11
gif dance_t5 0 10 12 --config "$cfg_stage" --theme dance --demo "pray:5:จักรพรรดิเพชร" --seed 55
gif dance_queue 0 9 12 --config "$cfg_floor" --theme dance --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:5:เศรษฐีต้น" --seed 42
gif dunk_t1 0 6 12 --config "$DCFG" --theme dunk --demo "pray:1:มะลิ" --seed 11
gif dunk_t5 0 9 12 --config "$DCFG" --theme dunk --demo "pray:5:จักรพรรดิเพชร" --seed 55
gif dunk_queue 0 7 12 --config "$DCFG" --theme dunk --demo "pray:1:มะลิ" --demo "pray:3:สมชาย" --demo "siamsi:2:นก" --seed 42

[ -n "${XPID:-}" ] && kill $XPID
echo done
ls -lh screenshots/dance/*.png screenshots/dunk/*.png previews/dance_*.gif previews/dunk_*.gif 2>/dev/null
