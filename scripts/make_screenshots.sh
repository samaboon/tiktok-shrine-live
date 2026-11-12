#!/usr/bin/env bash
# สร้างภาพตัวอย่างทั้งหมดแบบ headless (Linux + Xvfb) -> screenshots/
# เวลาในเกม: เดินเข้า 3.2 วิ | ไหว้พระ 3.4 วิ | เซียมซี 6.4 วิ | ลอยกระทง 4.3 วิ | อธิษฐาน 4.9 วิ | แสดงผล 5 + 0.5*ระดับ วิ
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
  timeout 120 "$BIN" --no-status --out "$OUT/raw/$name.png" --preview "$@" 2>&1 | grep -q saved || echo "FAILED $name"
  mv -f "$OUT/raw/${name}_checker.png" "$OUT/$name.png" 2>/dev/null
}
run() { shot "$@" & N=$((N+1)); if [ $((N % 6)) -eq 0 ]; then wait; fi; }
NAMES=("มะลิ" "สมชาย" "เศรษฐีต้น" "เจ้าสัวเฮง" "จักรพรรดิเพชร")
for t in 1 2 3 4 5; do
  nm=${NAMES[$((t-1))]}
  # ไหว้พระขอเลข
  run pray_t${t}_1_walk      --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 1.8
  run pray_t${t}_2_wai       --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 5.0
  run pray_t${t}_3_number    --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 9.2
  run pray_t${t}_4_fireworks --demo "pray:$t:$nm" --seed $((t*11)) --shot-at 10.2
  # เสี่ยงเซียมซี
  run siamsi_t${t}_1_shake   --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 5.0
  run siamsi_t${t}_2_stick   --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 9.0
  run siamsi_t${t}_3_fortune --demo "siamsi:$t:$nm" --seed $((t*13)) --shot-at 11.4
  # ลอยกระทง
  run krathong_t${t}_1_walk   --theme krathong --demo "pray:$t:$nm" --seed $((t*17)) --shot-at 1.8
  run krathong_t${t}_2_wish   --theme krathong --demo "pray:$t:$nm" --seed $((t*17)) --shot-at 5.0
  run krathong_t${t}_3_float  --theme krathong --demo "pray:$t:$nm" --seed $((t*17)) --shot-at 7.3
  run krathong_t${t}_4_number --theme krathong --demo "pray:$t:$nm" --seed $((t*17)) --shot-at 10.2
  run wish_t${t}_blessing     --theme krathong --demo "siamsi:$t:$nm" --seed $((t*19)) --shot-at 10.0
done
# ฉากเต็ม (scene) ของแต่ละโหมด
run scene_shrine_pray_t5     --bg scene --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2
run scene_shrine_siamsi_t3   --bg scene --demo "siamsi:3:เศรษฐีต้น" --seed 39 --shot-at 9.0
run scene_krathong_float_t4  --bg scene --theme krathong --demo "pray:4:เจ้าสัวเฮง" --seed 68 --shot-at 7.3
run scene_krathong_bless_t5  --bg scene --theme krathong --demo "siamsi:5:จักรพรรดิเพชร" --seed 95 --shot-at 10.0
# ตอนว่าง: ภาพนิ่ง (ไม่มีตัวละคร) vs กำลังเล่น
run idle_still_shrine        --shot-at 3
run idle_still_shrine_scene  --bg scene --shot-at 3
run idle_still_krathong      --theme krathong --shot-at 3
run idle_still_krathong_scene --bg scene --theme krathong --shot-at 3
run idle_after_krathongs_scene --bg scene --theme krathong --demo "pray:1:มะลิ" --demo "pray:3:ต้น" --demo "siamsi:5:เฮง" --seed 7 --shot-at 52
run playing_shrine_scene     --bg scene --demo "pray:3:เศรษฐีต้น" --seed 33 --shot-at 5.0
MULTI=(--demo "pray:3:มะลิ" --demo "siamsi:2:สมชาย" --demo "pray:4:ต้นข้าว" --demo "pray:1:Nok" --seed 5)
run multi_line_shrine_1      --bg scene "${MULTI[@]}" --shot-at 5.0
run multi_line_shrine_2      --bg scene "${MULTI[@]}" --shot-at 9.5
run multi_line_krathong      --bg scene --theme krathong "${MULTI[@]}" --shot-at 6.0
# โหมดพื้นหลัง/เลย์เอาต์
run mode_green_t5            --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2 --bg green
run mode_green_krathong_t5   --theme krathong --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2 --bg green
run layout_full_t5           --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2 --layout full
run layout_full_krathong_t5  --theme krathong --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2 --layout full --bg scene
run layout_top_t5            --demo "pray:5:จักรพรรดิเพชร" --seed 55 --shot-at 10.2 --layout top
wait
# settings window = separate OS window: grab it from the X screen (needs python3-pil)
sed -e 's/^x = .*/x = 1300/' -e 's/^check_on_start = .*/check_on_start = false/' config.toml > /tmp/_shotcfg.toml
setshot() { # name, args...
  local name=$1; shift
  "$BIN" --config /tmp/_shotcfg.toml --shot-at 99999 "$@" >/dev/null 2>&1 & local P=$!
  sleep 3
  python3 -c "from PIL import ImageGrab; ImageGrab.grab(xdisplay='$DISPLAY').crop((0,0,800,760)).save('$OUT/$name.png')"
  kill $P 2>/dev/null; wait $P 2>/dev/null
}
for i in 0 1 2 3 4 5 6 7; do setshot settings_tab$i --settings-tab $i; done
setshot settings_tab0_krathong --settings-tab 0 --theme krathong
setshot settings_tab3_krathong --settings-tab 3 --theme krathong
setshot settings_gift_picker --settings-tab 3
setshot settings_tab0_stopped --settings-tab 0 --stopped
[ -n "${XPID:-}" ] && kill $XPID
echo done
