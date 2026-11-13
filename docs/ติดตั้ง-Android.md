# ใช้มือถือ Android คู่กับ TikTok Shrine Live

> คู่มือนี้เขียนเมื่อ พ.ย. 2026 จากเอกสารทางการที่อ้างท้ายหน้า · เมนูในแอปอาจเปลี่ยนตามรุ่น · **ยังไม่ได้ทดสอบบนมือถือจริง** — ใช้เป็นแนวทางจากเอกสาร ไม่ใช่ผลทดลองในเครื่องจริง

## สำคัญ: ติดตั้งแอปนี้บนมือถือไม่ได้

**TikTok Shrine Live รันได้เฉพาะบนคอมพิวเตอร์ Windows หรือ Mac** (มีไฟล์ Linux สำหรับผู้ที่คอมไพล์เอง) ไม่มีแอปใน Google Play และติดตั้งบน Android ไม่ได้

มือถือใช้ได้ 2 ทางด้านล่าง — ทั้งสองทางต้องมีคอมพิวเตอร์ที่เปิดโปรแกรมนี้ไว้

| ทาง | ผู้ชมเห็นเอฟเฟกต์ทับภาพไลฟ์หรือไม่ |
|---|---|
| **ก) มือถือเป็นกล้องของคอม แล้วไลฟ์จากคอม** | เห็น — เพราะภาพที่ออกจาก OBS / TikTok LIVE Studio รวมเอฟเฟกต์ไว้แล้ว |
| **ข) ไลฟ์จากแอป TikTok บนมือถือ โดยคอมเปิดโปรแกรมค้างไว้** | **ไม่เห็น** — โปรแกรมบนคอมยังตอบของขวัญตามชื่อผู้ใช้ได้ แต่ผู้ชมไลฟ์ที่ออกจากมือถืออย่างเดียวจะไม่เห็นโอเวอร์เลย์ ถ้าอยากให้ผู้ชมเห็น ต้องไลฟ์จากคอมตามทาง ก |

## ก) มือถือเป็นกล้องของคอม แล้วไลฟ์จากคอม (แนะนำถ้าอยากให้ผู้ชมเห็นเอฟเฟกต์)

ต้องใช้คอมพิวเตอร์ Windows หรือ Mac ตามคู่มือของเครื่องนั้นก่อน: [ติดตั้งบน Windows](ติดตั้ง-Windows.md) หรือ [ติดตั้งบน Mac](ติดตั้ง-Mac.md)

จากนั้นให้มือถือเป็นแหล่งกล้องของโปรแกรมไลฟ์ (OBS หรือ TikTok LIVE Studio) แล้วจัดฉากให้กล้องอยู่ล่าง หน้าต่างเอฟเฟกต์อยู่บน

### ทางเลือกกล้องบน Windows

- **Phone Link (มีใน Windows 11 ไม่ต้องลงโปรแกรมเพิ่มบนคอม)** ตาม [เอกสาร Microsoft](https://support.microsoft.com/en-us/windows/apps/phonelink/use-your-mobile-device-s-camera): Windows 11, Android 10 ขึ้นไป, แอป **Link to Windows** บนมือถือ (รุ่น 1.24022.0 ขึ้นไป) → บนคอมไปที่ **Settings → Bluetooth & devices → Mobile devices** เชื่อมมือถือ (บางรุ่นต้องเปิดสวิตช์อนุญาตและกด **Manage devices** ก่อน) → เลือกมือถือแล้วเปิด **Use as connected camera** → อนุญาตกล้องในมือถือ → ใน OBS เพิ่ม **Video Capture Device** แล้วเลือกกล้องมือถือ
- **Camo**: แอป **Camo** บนมือถือ + **Camo Studio** บน Windows → เชื่อมด้วยสาย USB หรือสแกน QR ผ่าน Wi-Fi → ใน OBS เลือกกล้อง **Camo Camera** (ต่อสาย USB บน Android ต้องเปิด USB debugging) ดู [Camo getting started](https://camo.com/support/camo/camo-getting-started)
- **DroidCam**: แอป DroidCam บนมือถือ + [ปลั๊กอิน DroidCam สำหรับ OBS](https://droidcam.app/obs/) ให้ตรงรุ่น OBS → ใน OBS เพิ่ม Source **DroidCam** → **Use WiFi** ใส่ที่อยู่ IP ที่แอปแสดง → **Activate** (หรือใช้ DroidCam Windows Client แล้วเลือกกล้อง **DroidCam Webcam**)

รายละเอียดจัดฉากและใส่หน้าต่างเอฟเฟกต์: [ใช้มือถือเป็นกล้องบน Windows](ติดตั้ง-Windows.md#7-ใช้มือถือเป็นกล้อง)

### ทางเลือกกล้องบน Mac

- **Camo**: แอป **Camo Camera** บนมือถือ (Google Play) + **Camo Studio** บน Mac (เปิดครั้งแรกให้ติดตั้ง camera extension และอนุญาตใน System Settings) → เสียบสายหรือสแกน QR ผ่าน Wi-Fi → ใน OBS เลือกกล้อง **Camo Camera** ดู [Camo: ใช้ Android เป็นเว็บแคม](https://camo.com/support/how-to/use-android-as-webcam)
- **DroidCam**: แอปบนมือถือ + [ปลั๊กอิน DroidCam สำหรับ OBS](https://droidcam.app/obs/) → ใน OBS เพิ่ม Source **DroidCam** → **Use WiFi** ใส่ที่อยู่ IP ที่แอปแสดง → **Activate** และอนุญาต OBS ใน **System Settings → Privacy & Security → Local Network**

รายละเอียดจัดฉาก: [ใช้มือถือเป็นกล้องบน Mac](ติดตั้ง-Mac.md#7-ใช้มือถือเป็นกล้อง)

### ใส่กล้องใน OBS / TikTok LIVE Studio

- ใน OBS: **Sources → + → Video Capture Device** แล้วเลือกกล้องที่ขึ้นจากวิธีด้านบน ([เอกสาร OBS: Video Capture Sources](https://obsproject.com/kb/video-capture-sources)) · ให้ Source กล้องอยู่ล่าง (คลิกขวา → Transform → Fit to screen) และ Source หน้าต่างเอฟเฟกต์อยู่บน
- ใน TikTok LIVE Studio: ดาวน์โหลดจาก [หน้าดาวน์โหลดทางการ](https://www.tiktok.com/studio/download) แล้วเพิ่มแหล่งกล้องตาม [บทความเพิ่มกล้องของ LIVE Studio](https://www.tiktok.com/live/studio/help/article/Get-started-with-your-first-LIVE/Add-a-camera-source-to-let-viewers-know-you) · ถ้าพื้นหลังหน้าต่างเอฟเฟกต์ไม่ใส ให้ทำภาพใน OBS แล้วใช้ **OBS Virtual Camera** ใน LIVE Studio

เมื่อฉากพร้อมแล้ว **ไลฟ์จากคอม** (OBS + Stream Key จาก [LIVE Center](https://livecenter.tiktok.com) หรือกด Go LIVE ใน LIVE Studio) จากนั้นในโปรแกรมนี้กด **เชื่อมต่อ** ด้วยชื่อผู้ใช้ TikTok เดียวกัน

## ข) ไลฟ์จากแอป TikTok บนมือถือ — โปรแกรมบนคอมยังรับของขวัญได้ แต่ผู้ชมไม่เห็นเอฟเฟกต์

ใช้ทางนี้เมื่ออยากไลฟ์จากแอปมือถือตามปกติ และให้โปรแกรมบนคอมทำงานตามของขวัญ (เช่น ดูเอง หรือบันทึกหน้าจอคอมแยก) **ผู้ชมที่เข้าไลฟ์จากมือถือจะไม่เห็นโอเวอร์เลย์** เว้นแต่ภาพที่ออกไปเป็นภาพจากคอมตามทาง ก

1. เปิด TikTok Shrine Live บนคอม → ใส่**ชื่อผู้ใช้ TikTok เดียวกับบัญชีที่จะไลฟ์** → บันทึก → กด **เชื่อมต่อ**
2. บนมือถือ เริ่มไลฟ์ตาม [คู่มือทางการของ TikTok (29 ก.ย. 2026)](https://www.tiktok.com/live/creator-networks/en/blog/how-to-go-live-on-tiktok) และ [หน้าช่วยเหลือ TikTok LIVE](https://support.tiktok.com/en/live-gifts-wallet/tiktok-live/what-is-tiktok-live):
   1. เปิดแอป TikTok แล้วแตะปุ่ม **Create (+)** / **Add post +** ที่ด้านล่าง
   2. เลื่อนตัวเลือกด้านล่างแล้วเลือก **LIVE** (บางรุ่นต้องเลื่อนหา)
   3. ใส่ชื่อเรื่อง รูปปก หรือปรับการตั้งค่าอื่นตามต้องการ
   4. แตะ **Go LIVE** เพื่อเริ่ม
3. เมื่อไลฟ์ขึ้นและโปรแกรมบนคอมแสดงสถานะ **เชื่อมต่อแล้ว** การส่งของขวัญในไลฟ์จะทำให้เอฟเฟกต์ทำงานบนหน้าต่างคอม — แต่ภาพนั้นไม่ถูกส่งเข้าไลฟ์มือถือโดยอัตโนมัติ

บัญชีที่ไลฟ์ได้ต้องมีอายุอย่างน้อย 18 ปี และมีจำนวนผู้ติดตามตามเกณฑ์ของภูมิภาค (อาจต่างกัน) ตามเอกสารทางการด้านบน · ยังไม่ได้ทดสอบขั้นตอนนี้บนมือถือจริง

## ข้อมูลอ้างอิง

ตรวจเมื่อ พ.ย. 2026 เมนูอาจเปลี่ยนตามรุ่น:

- [TikTok: How to go LIVE on TikTok (29 ก.ย. 2026)](https://www.tiktok.com/live/creator-networks/en/blog/how-to-go-live-on-tiktok)
- [TikTok Support: What is TikTok LIVE](https://support.tiktok.com/en/live-gifts-wallet/tiktok-live/what-is-tiktok-live)
- [ดาวน์โหลด TikTok LIVE Studio](https://www.tiktok.com/studio/download) · [เพิ่มแหล่งกล้องใน LIVE Studio](https://www.tiktok.com/live/studio/help/article/Get-started-with-your-first-LIVE/Add-a-camera-source-to-let-viewers-know-you)
- [OBS: Video Capture Sources](https://obsproject.com/kb/video-capture-sources)
- [Microsoft: ใช้กล้องมือถือผ่าน Phone Link](https://support.microsoft.com/en-us/windows/apps/phonelink/use-your-mobile-device-s-camera)
- [DroidCam สำหรับ OBS](https://droidcam.app/obs/) · [Camo](https://camo.com/support/camo/camo-getting-started)
- คู่มือคอม: [Windows](ติดตั้ง-Windows.md) · [Mac](ติดตั้ง-Mac.md) · [คู่มือตั้งค่า](คู่มือ.md)
