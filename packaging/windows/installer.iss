; TikTok Shrine Live — one-click per-user installer (Inno Setup 6, Unicode)
; Build on the CI windows runner AFTER packaging/make_windows_zip.sh, which stages
; dist\TikTokShrineLive\ with exactly the zip contents (exe + config + docs + runtime).
;   ISCC.exe /DMyAppVersion=0.2.5 packaging\windows\installer.iss
; Output: dist\TikTokShrineLive-Setup-x64.exe

#ifndef MyAppVersion
  #define MyAppVersion "0.2.5"
#endif

[Setup]
AppId={{7C3E9A52-4B8D-4E6F-9A27-D5F1C8B3E641}
AppName=TikTok Shrine Live
AppVersion={#MyAppVersion}
AppPublisher=samaboon
AppPublisherURL=https://github.com/samaboon/tiktok-shrine-live
AppSupportURL=https://github.com/samaboon/tiktok-shrine-live/issues
AppUpdatesURL=https://github.com/samaboon/tiktok-shrine-live/releases/latest
DefaultDirName={localappdata}\Programs\TikTokShrineLive
DefaultGroupName=TikTok Shrine Live
DisableProgramGroupPage=yes
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
OutputDir=..\..\dist
OutputBaseFilename=TikTokShrineLive-Setup-x64
SetupIconFile=..\..\assets\icon\icon.ico
UninstallDisplayIcon={app}\TikTokShrineLive.exe
UninstallDisplayName=TikTok Shrine Live
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
VersionInfoVersion={#MyAppVersion}
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

; Thai audience: keep the English wizard chrome, speak Thai on the big pages.
[Messages]
WelcomeLabel1=ยินดีต้อนรับสู่ TikTok Shrine Live
WelcomeLabel2=โปรแกรมเอฟเฟกต์ไลฟ์ TikTok: ไหว้พระ เซียมซี ลอยกระทง ให้อาหารแมว เต้น ปาตกน้ำ และเลี้ยงแมว%n%nตัวติดตั้งรวมตัวรันตัวเชื่อม TikTok มาให้ ไม่ต้องติดตั้ง Node.js เพิ่ม%n%nกด Next เพื่อเริ่มติดตั้ง
FinishedHeadingLabel=ติดตั้งเสร็จแล้ว
FinishedLabel=เปิดโปรแกรมจาก shortcut ที่สร้างไว้%n%nครั้งแรก Windows อาจขึ้นหน้าจอสีน้ำเงิน "Windows protected your PC" ให้กด More info แล้ว Run anyway (โปรแกรมไม่ได้ซื้อใบรับรองดิจิทัล ไม่ใช่ไวรัส)
FinishedLabelNoIcons=เปิดโปรแกรมจาก Start Menu หรือโฟลเดอร์ที่ติดตั้ง%n%nครั้งแรก Windows อาจขึ้นหน้าจอสีน้ำเงิน "Windows protected your PC" ให้กด More info แล้ว Run anyway (โปรแกรมไม่ได้ซื้อใบรับรองดิจิทัล ไม่ใช่ไวรัส)

[Tasks]
Name: "desktopicon"; Description: "สร้าง shortcut บน Desktop"; GroupDescription: "shortcut เพิ่มเติม:"; Flags: unchecked

[Files]
; mirror the portable zip exactly; never overwrite an existing user config on upgrade
Source: "..\..\dist\TikTokShrineLive\*"; Excludes: "config.toml"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs ignoreversion
Source: "..\..\dist\TikTokShrineLive\config.toml"; DestDir: "{app}"; Flags: onlyifdoesntexist uninsneveruninstall

[Icons]
Name: "{group}\TikTok Shrine Live"; Filename: "{app}\TikTokShrineLive.exe"
Name: "{group}\คู่มือการใช้งาน"; Filename: "{app}\คู่มือ.md"
Name: "{group}\ถอนการติดตั้ง TikTok Shrine Live"; Filename: "{uninstallexe}"
Name: "{autodesktop}\TikTok Shrine Live"; Filename: "{app}\TikTokShrineLive.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\TikTokShrineLive.exe"; Description: "เปิด TikTok Shrine Live ตอนนี้"; Flags: nowait postinstall skipifsilent shellexec