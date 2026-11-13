#!/bin/bash
# เปิดครั้งแรก — ปลดล็อก Gatekeeper (quarantine) ให้ TikTok Shrine Live แล้วเปิดแอป
# ใช้ครั้งเดียว: ลากแอปไป Applications ก่อน แล้วดับเบิลคลิกไฟล์นี้จากหน้าต่าง dmg
# (ถ้าไฟล์นี้เองถูกบล็อก: คลิกขวา → Open)
cd "$(dirname "$0")" || exit 1

APP=""
for cand in "/Applications/TikTok Shrine Live.app" "$HOME/Applications/TikTok Shrine Live.app" "./TikTok Shrine Live.app"; do
  if [ -d "$cand" ]; then APP="$cand"; break; fi
done

if [ -z "$APP" ]; then
  echo "ไม่พบ TikTok Shrine Live.app"
  echo "ลากแอปจากหน้าต่างนี้ไปวางบนโฟลเดอร์ Applications ก่อน แล้วดับเบิลคลิกไฟล์นี้อีกครั้ง"
  read -r -p "กด Enter เพื่อปิดหน้าต่างนี้..." _
  exit 1
fi

echo "กำลังปลดล็อกการกักกัน (quarantine): $APP"
xattr -dr com.apple.quarantine "$APP" 2>/dev/null || true
echo "กำลังเปิดแอป..."
open "$APP"
echo ""
echo "ถ้ายังไม่เปิด: คลิกขวาที่แอป → Open"
echo "หรือ macOS 15 ขึ้นไป: System Settings → Privacy & Security → Open Anyway"
read -r -p "กด Enter เพื่อปิดหน้าต่างนี้..." _