//! In-app updates.
//! 1) App update: reads the latest GitHub Release (public API), downloads the asset for this OS,
//!    swaps the running executable / .app bundle and restarts.
//! 2) TikTok bridge update: replaces only the tiktok-live-connector bundle (bridge.cjs) from a
//!    release asset, or falls back to `npm install tiktok-live-connector@latest` - no app release needed.
//! Uses the system `curl` (built into Windows 10+, macOS and Linux) so no TLS stack is compiled in.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub const BRIDGE_SOURCE: &str = include_str!("../bridge/bridge.js");
pub const BRIDGE_BUILTIN_VERSION_JSON: &str = include_str!("../bridge/dist/bridge-version.json");

#[derive(Clone, Debug, Default)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Clone, Debug, Default)]
pub struct ReleaseInfo {
    pub tag: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<Asset>,
}

#[derive(Clone, Debug, Default)]
pub struct UpState {
    pub current: String,
    pub checking: bool,
    pub checked: bool,
    pub latest: Option<ReleaseInfo>,
    pub available: bool,
    pub error: String,
    pub busy: bool,
    pub phase: String,
    pub progress: Option<(u64, u64)>,
    /// set when the new binary is in place: the UI spawns it and quits
    pub restart: Option<PathBuf>,
    // bridge
    pub bridge_current: String,
    pub bridge_release: Option<String>,
    pub bridge_npm: Option<String>,
    pub bridge_checking: bool,
    pub bridge_busy: bool,
    pub bridge_msg: String,
    /// new bridge script ready -> app restarts the bridge with it
    pub bridge_new_script: Option<PathBuf>,
}

#[derive(Clone)]
pub struct Updater {
    pub st: Arc<Mutex<UpState>>,
    pub repo: String,
    pub api_base: String,
    pub dl_base: String,
    pub bridge_tag: String,
    pub dir: PathBuf,
}

/// "v0.2.10" -> [0,2,10]
pub fn parse_ver(s: &str) -> Vec<u64> {
    s.trim().trim_start_matches(['v', 'V']).split(|c: char| c == '.' || c == '-' || c == '+').take(3).map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().unwrap_or(0)).collect()
}
pub fn newer(a: &str, b: &str) -> bool {
    let (mut x, mut y) = (parse_ver(a), parse_ver(b));
    x.resize(3, 0);
    y.resize(3, 0);
    x > y
}

fn no_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let _ = cmd;
}

fn curl() -> Command {
    let mut c = Command::new(if cfg!(windows) { "curl.exe" } else { "curl" });
    c.args(["-fsSL", "--retry", "2", "--connect-timeout", "15", "-A", "tiktok-shrine-live-updater"]);
    c.stdin(Stdio::null());
    no_window(&mut c);
    c
}

pub fn fetch_text(url: &str) -> Result<String, String> {
    let out = curl().args(["--max-time", "30", "-H", "Accept: application/vnd.github+json", url]).output().map_err(|e| format!("เรียก curl ไม่ได้: {e}"))?;
    if !out.status.success() {
        return Err(format!("ดาวน์โหลดไม่สำเร็จ ({}) {}", out.status, String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Download with progress callback (polls the partial file size).
pub fn download(url: &str, dest: &Path, size: u64, mut progress: impl FnMut(u64, u64)) -> Result<(), String> {
    let part = dest.with_extension("part");
    let _ = std::fs::remove_file(&part);
    let mut child = curl().args(["--max-time", "900", "-o"]).arg(&part).arg(url).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().map_err(|e| format!("เรียก curl ไม่ได้: {e}"))?;
    loop {
        let got = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
        progress(got, size);
        match child.try_wait() {
            Ok(Some(st)) => {
                if !st.success() {
                    let mut err = String::new();
                    if let Some(mut e) = child.stderr.take() {
                        use std::io::Read;
                        let _ = e.read_to_string(&mut err);
                    }
                    return Err(format!("ดาวน์โหลดไม่สำเร็จ: {}", err.trim()));
                }
                break;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(150)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let got = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if size > 0 && got != size {
        return Err(format!("ไฟล์ไม่ครบ ({got}/{size} ไบต์)"));
    }
    progress(got, got);
    std::fs::rename(&part, dest).map_err(|e| e.to_string())
}

pub fn parse_release(v: &Value) -> Option<ReleaseInfo> {
    let tag = v.get("tag_name")?.as_str()?.to_string();
    let assets = v.get("assets").and_then(|a| a.as_array()).map(|a| {
        a.iter().filter_map(|x| Some(Asset { name: x.get("name")?.as_str()?.to_string(), url: x.get("browser_download_url")?.as_str()?.to_string(), size: x.get("size").and_then(|s| s.as_u64()).unwrap_or(0) })).collect()
    }).unwrap_or_default();
    Some(ReleaseInfo {
        tag,
        name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        body: v.get("body").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        html_url: v.get("html_url").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        assets,
    })
}

/// Pick the release asset for the running OS.
pub fn pick_asset(assets: &[Asset]) -> Option<Asset> {
    let low = |a: &Asset| a.name.to_lowercase();
    let find = |f: &dyn Fn(&str) -> bool| assets.iter().find(|a| f(&low(a))).cloned();
    if cfg!(windows) {
        find(&|n| n.contains("windows") && n.ends_with(".zip")).or_else(|| find(&|n| n.ends_with(".exe")))
    } else if cfg!(target_os = "macos") {
        find(&|n| n.contains("macos") && n.ends_with(".zip")).or_else(|| find(&|n| n.ends_with(".dmg")))
    } else {
        find(&|n| n.contains("linux") && (n.ends_with(".tar.gz") || n.ends_with(".zip")))
    }
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    no_window(cmd);
    let out = cmd.stdin(Stdio::null()).output().map_err(|e| format!("{what}: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(format!("{what}: {}", String::from_utf8_lossy(&out.stderr).trim())) }
}

fn find_file(dir: &Path, pred: &dyn Fn(&Path) -> bool, depth: usize) -> Option<PathBuf> {
    let rd = std::fs::read_dir(dir).ok()?;
    let mut subdirs = vec![];
    for e in rd.flatten() {
        let p = e.path();
        if pred(&p) {
            return Some(p);
        }
        if p.is_dir() {
            subdirs.push(p);
        }
    }
    if depth == 0 {
        return None;
    }
    subdirs.into_iter().find_map(|d| find_file(&d, pred, depth - 1))
}

fn extract(file: &Path, into: &Path) -> Result<(), String> {
    let name = file.to_string_lossy().to_lowercase();
    std::fs::create_dir_all(into).map_err(|e| e.to_string())?;
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        run(Command::new("tar").arg("-xzf").arg(file).arg("-C").arg(into), "แตกไฟล์")
    } else if name.ends_with(".zip") {
        if cfg!(target_os = "macos") {
            run(Command::new("ditto").arg("-x").arg("-k").arg(file).arg(into), "แตกไฟล์")
        } else if cfg!(windows) {
            run(Command::new("tar.exe").arg("-xf").arg(file).arg("-C").arg(into), "แตกไฟล์")
                .or_else(|_| run(Command::new("powershell").args(["-NoProfile", "-Command"]).arg(format!("Expand-Archive -Force -LiteralPath '{}' -DestinationPath '{}'", file.display(), into.display())), "แตกไฟล์"))
        } else {
            run(Command::new("unzip").arg("-o").arg("-q").arg(file).arg("-d").arg(into), "แตกไฟล์")
        }
    } else if name.ends_with(".dmg") {
        let mnt = into.join("mnt");
        std::fs::create_dir_all(&mnt).map_err(|e| e.to_string())?;
        run(Command::new("hdiutil").args(["attach", "-nobrowse", "-readonly", "-mountpoint"]).arg(&mnt).arg(file), "เปิด dmg")?;
        let app = find_file(&mnt, &|p| p.extension().map(|e| e == "app").unwrap_or(false), 1);
        let r = match app {
            Some(a) => run(Command::new("ditto").arg(&a).arg(into.join(a.file_name().unwrap())), "คัดลอกแอป"),
            None => Err("ไม่พบ .app ใน dmg".into()),
        };
        let _ = run(Command::new("hdiutil").arg("detach").arg(&mnt).arg("-quiet"), "ปิด dmg");
        r
    } else {
        // plain executable
        let dst = into.join(file.file_name().unwrap());
        std::fs::copy(file, dst).map(|_| ()).map_err(|e| e.to_string())
    }
}

/// Remove `*.old` leftovers of a previous self-update.
pub fn cleanup_old() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_path(&exe));
        if let Some(app) = mac_bundle(&exe) {
            let _ = std::fs::remove_dir_all(app.with_extension("app.old"));
        }
    }
}
fn old_path(exe: &Path) -> PathBuf {
    let mut s = exe.as_os_str().to_owned();
    s.push(".old");
    PathBuf::from(s)
}
fn mac_bundle(exe: &Path) -> Option<PathBuf> {
    exe.ancestors().find(|p| p.extension().map(|e| e == "app").unwrap_or(false)).map(|p| p.to_path_buf())
}

/// Launch a freshly installed binary with `--selftest` (no window) and wait up to `timeout`.
/// Catches binaries that cannot start on this machine (e.g. missing glibc version, wrong arch, corrupt file).
pub fn selftest(exe: &Path, timeout: std::time::Duration) -> Result<(), String> {
    let mut cmd = Command::new(exe);
    cmd.arg("--selftest").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    no_window(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("เปิดไฟล์ใหม่ไม่ได้: {e}"))?;
    let t0 = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if t0.elapsed() < timeout => std::thread::sleep(std::time::Duration::from_millis(100)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("ไฟล์ใหม่ไม่ตอบสนองภายใน {} วินาที", timeout.as_secs()));
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    let so = String::from_utf8_lossy(&out.stdout);
    if out.status.success() && so.contains("selftest ok") {
        eprintln!("[update] ทดสอบไฟล์ใหม่ผ่าน: {}", so.trim());
        Ok(())
    } else {
        let se = String::from_utf8_lossy(&out.stderr);
        let msg = format!("{} {}", so.trim(), se.trim()).trim().chars().take(300).collect::<String>();
        Err(format!("ไฟล์ใหม่เปิดไม่ผ่าน ({}): {msg}", out.status))
    }
}

/// Install the downloaded asset over the running app, verify it launches (`--selftest`),
/// and roll back to the previous version if it does not. Returns the executable to restart.
pub fn install(file: &Path, work: &Path) -> Result<PathBuf, String> {
    install_verified(file, work, &|exe| selftest(exe, std::time::Duration::from_secs(20)))
}

pub fn install_verified(file: &Path, work: &Path, verify: &dyn Fn(&Path) -> Result<(), String>) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let ex = work.join("x");
    let _ = std::fs::remove_dir_all(&ex);
    extract(file, &ex)?;
    if cfg!(target_os = "macos") {
        if let Some(cur_app) = mac_bundle(&exe) {
            let new_app = find_file(&ex, &|p| p.extension().map(|e| e == "app").unwrap_or(false), 2).ok_or("ไม่พบ .app ในไฟล์อัปเดต")?;
            let old = cur_app.with_extension("app.old");
            let _ = std::fs::remove_dir_all(&old);
            std::fs::rename(&cur_app, &old).map_err(|e| format!("ย้ายแอปเดิมไม่ได้: {e}"))?;
            if let Err(e) = run(Command::new("ditto").arg(&new_app).arg(&cur_app), "ติดตั้งแอปใหม่") {
                let _ = std::fs::rename(&old, &cur_app);
                return Err(e);
            }
            let _ = run(Command::new("xattr").args(["-dr", "com.apple.quarantine"]).arg(&cur_app), "xattr");
            let rel = exe.strip_prefix(&cur_app).map(|p| p.to_path_buf()).unwrap_or_default();
            let new_exe = cur_app.join(rel);
            if let Err(e) = verify(&new_exe) {
                let _ = std::fs::remove_dir_all(&cur_app);
                let _ = std::fs::rename(&old, &cur_app);
                eprintln!("[update] {e} -> ย้อนกลับเป็นเวอร์ชันเดิม");
                return Err(format!("{e} — ย้อนกลับเป็นเวอร์ชันเดิมแล้ว"));
            }
            return Ok(new_exe);
        }
    }
    let exe_name = exe.file_name().unwrap().to_string_lossy().to_string();
    let want: Vec<String> = vec![exe_name.to_lowercase(), "tiktokshrinelive.exe".into(), "tiktok-shrine-live".into()];
    let new_bin = find_file(&ex, &|p| p.is_file() && p.file_name().map(|n| want.contains(&n.to_string_lossy().to_lowercase())).unwrap_or(false), 3).ok_or("ไม่พบไฟล์โปรแกรมในไฟล์อัปเดต")?;
    let old = old_path(&exe);
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| format!("เปลี่ยนชื่อไฟล์เดิมไม่ได้: {e}"))?;
    if let Err(e) = std::fs::copy(&new_bin, &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(format!("คัดลอกไฟล์ใหม่ไม่ได้: {e}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755));
    }
    if let Err(e) = verify(&exe) {
        // put the old binary back; the running (old) version simply keeps going
        let _ = std::fs::remove_file(&exe);
        let _ = std::fs::rename(&old, &exe);
        let _ = std::fs::remove_dir_all(&ex);
        eprintln!("[update] {e} -> ย้อนกลับเป็นเวอร์ชันเดิม");
        return Err(format!("{e} — ย้อนกลับเป็นเวอร์ชันเดิมแล้ว"));
    }
    // refresh the bundled manual next to the exe (config.toml is never touched)
    if let Some(d) = exe.parent() {
        if let Some(m) = find_file(&ex, &|p| p.file_name().map(|n| n == "คู่มือ.md").unwrap_or(false), 3) {
            let _ = std::fs::copy(m, d.join("คู่มือ.md"));
        }
    }
    let _ = std::fs::remove_dir_all(&ex);
    Ok(exe)
}

/// Bridge override written by a bridge update: (script, connector version)
pub fn bridge_override(dir: &Path) -> Option<(PathBuf, String)> {
    let meta: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("bridge").join("bridge-update.json")).ok()?).ok()?;
    let ver = meta.get("connector")?.as_str()?.to_string();
    let script = dir.join(meta.get("script")?.as_str()?);
    if !script.exists() || !newer(&ver, &builtin_bridge_version()) {
        return None;
    }
    Some((script, ver))
}
pub fn builtin_bridge_version() -> String {
    serde_json::from_str::<Value>(BRIDGE_BUILTIN_VERSION_JSON).ok().and_then(|v| v.get("connector").and_then(|s| s.as_str()).map(String::from)).unwrap_or_else(|| "?".into())
}

impl Updater {
    pub fn new(repo: &str, api_base: &str, dl_base: &str, bridge_tag: &str, dir: PathBuf, current: String, bridge_current: String) -> Self {
        let st = UpState { current, bridge_current, ..Default::default() };
        Self { st: Arc::new(Mutex::new(st)), repo: repo.into(), api_base: api_base.trim_end_matches('/').into(), dl_base: dl_base.trim_end_matches('/').into(), bridge_tag: bridge_tag.into(), dir }
    }
    pub fn snapshot(&self) -> UpState {
        self.st.lock().unwrap().clone()
    }
    pub fn snapshot_json(&self) -> Value {
        let s = self.snapshot();
        serde_json::json!({
            "current": s.current, "checked": s.checked, "checking": s.checking, "available": s.available,
            "latest": s.latest.as_ref().map(|l| l.tag.clone()), "error": s.error, "busy": s.busy, "phase": s.phase,
            "progress": s.progress.map(|(a, b)| [a, b]),
            "bridge_current": s.bridge_current, "bridge_release": s.bridge_release, "bridge_npm": s.bridge_npm,
            "bridge_busy": s.bridge_busy, "bridge_msg": s.bridge_msg,
        })
    }

    /// Check app + bridge versions in the background.
    pub fn check(&self) {
        {
            let mut s = self.st.lock().unwrap();
            if s.checking {
                return;
            }
            s.checking = true;
            s.bridge_checking = true;
            s.error.clear();
        }
        let me = self.clone();
        std::thread::spawn(move || {
            let url = format!("{}/repos/{}/releases/latest", me.api_base, me.repo);
            let res = fetch_text(&url).and_then(|t| serde_json::from_str::<Value>(&t).map_err(|e| e.to_string())).and_then(|v| parse_release(&v).ok_or_else(|| "อ่านข้อมูลเวอร์ชันไม่ได้".to_string()));
            {
                let mut s = me.st.lock().unwrap();
                s.checking = false;
                s.checked = true;
                match &res {
                    Ok(r) => {
                        s.available = newer(&r.tag, &s.current);
                        s.latest = Some(r.clone());
                        eprintln!("[update] ล่าสุด {} (ตอนนี้ {}) -> {}", r.tag, s.current, if s.available { "มีอัปเดต" } else { "ล่าสุดแล้ว" });
                    }
                    Err(e) => {
                        s.error = e.clone();
                        eprintln!("[update] ตรวจไม่ได้: {e}");
                    }
                }
            }
            me.check_bridge(res.ok());
        });
    }

    fn check_bridge(&self, rel: Option<ReleaseInfo>) {
        // 1) rolling "bridge-latest" release, 2) bridge-version.json in the latest app release
        let mut rel_ver = None;
        let rolling = format!("{}/{}/releases/download/{}/bridge-version.json", self.dl_base, self.repo, self.bridge_tag);
        let mut sources = vec![rolling];
        if let Some(r) = &rel {
            if let Some(a) = r.assets.iter().find(|a| a.name == "bridge-version.json") {
                sources.push(a.url.clone());
            }
        }
        for u in sources {
            if let Ok(t) = fetch_text(&u) {
                if let Some(v) = serde_json::from_str::<Value>(&t).ok().and_then(|v| v.get("connector").and_then(|s| s.as_str()).map(String::from)) {
                    if rel_ver.as_ref().map(|r: &String| newer(&v, r)).unwrap_or(true) {
                        rel_ver = Some(v);
                    }
                }
            }
        }
        let npm = fetch_text("https://registry.npmjs.org/tiktok-live-connector/latest").ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from));
        let mut s = self.st.lock().unwrap();
        s.bridge_release = rel_ver;
        s.bridge_npm = npm;
        s.bridge_checking = false;
        eprintln!("[update] ตัวเชื่อม TikTok: ใช้อยู่ {} | release {:?} | npm {:?}", s.bridge_current, s.bridge_release, s.bridge_npm);
    }

    /// Download + install the app update for this OS.
    pub fn apply(&self) {
        let (rel, busy) = {
            let s = self.st.lock().unwrap();
            (s.latest.clone(), s.busy)
        };
        let Some(rel) = rel else { return };
        if busy {
            return;
        }
        let me = self.clone();
        std::thread::spawn(move || {
            let set = |phase: &str, err: Option<String>| {
                let mut s = me.st.lock().unwrap();
                s.phase = phase.to_string();
                if let Some(e) = err {
                    s.error = e;
                    s.busy = false;
                }
            };
            me.st.lock().unwrap().busy = true;
            let Some(asset) = pick_asset(&rel.assets) else {
                set("", Some(format!("ยังไม่มีไฟล์สำหรับเครื่องนี้ใน {} - ดาวน์โหลดเองที่ {}", rel.tag, rel.html_url)));
                return;
            };
            let work = me.dir.join("update-tmp");
            let _ = std::fs::remove_dir_all(&work);
            let _ = std::fs::create_dir_all(&work);
            let file = work.join(&asset.name);
            set(&format!("กำลังดาวน์โหลด {}", asset.name), None);
            let st = me.st.clone();
            if let Err(e) = download(&asset.url, &file, asset.size, |a, b| st.lock().unwrap().progress = Some((a, b))) {
                set("", Some(e));
                return;
            }
            set("กำลังติดตั้ง...", None);
            match install(&file, &work) {
                Ok(exe) => {
                    let _ = std::fs::remove_dir_all(&work);
                    let mut s = me.st.lock().unwrap();
                    s.phase = format!("ติดตั้ง {} แล้ว กำลังเปิดใหม่...", rel.tag);
                    s.restart = Some(exe);
                }
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&work);
                    set("", Some(format!("ติดตั้งไม่สำเร็จ: {e}")));
                }
            }
        });
    }

    /// Update only the TikTok connector bundle.
    pub fn update_bridge(&self, node_cmd: String) {
        if self.st.lock().unwrap().bridge_busy {
            return;
        }
        self.st.lock().unwrap().bridge_busy = true;
        let me = self.clone();
        std::thread::spawn(move || {
            let r = me.update_bridge_inner(&node_cmd);
            let mut s = me.st.lock().unwrap();
            s.bridge_busy = false;
            match r {
                Ok((script, ver, msg)) => {
                    s.bridge_current = ver;
                    s.bridge_msg = msg;
                    s.bridge_new_script = Some(script);
                }
                Err(e) => s.bridge_msg = e,
            }
            eprintln!("[update] ตัวเชื่อม: {}", s.bridge_msg);
        });
    }

    fn update_bridge_inner(&self, node_cmd: &str) -> Result<(PathBuf, String, String), String> {
        let cur = self.snapshot().bridge_current;
        let bdir = self.dir.join("bridge");
        std::fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
        self.st.lock().unwrap().bridge_msg = "กำลังตรวจเวอร์ชัน...".into();
        let latest_rel = fetch_text(&format!("{}/repos/{}/releases/latest", self.api_base, self.repo)).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| parse_release(&v));
        self.check_bridge(latest_rel.clone());
        let s = self.snapshot();
        let rel_v = s.bridge_release.clone().unwrap_or_default();
        let npm_v = s.bridge_npm.clone().unwrap_or_default();
        // A) prebuilt bundle from a release
        if !rel_v.is_empty() && newer(&rel_v, &cur) && (npm_v.is_empty() || !newer(&npm_v, &rel_v)) {
            self.st.lock().unwrap().bridge_msg = format!("กำลังดาวน์โหลดตัวเชื่อม {rel_v}...");
            let mut urls = vec![format!("{}/{}/releases/download/{}/bridge.cjs", self.dl_base, self.repo, self.bridge_tag)];
            if let Some(a) = latest_rel.as_ref().and_then(|r| r.assets.iter().find(|a| a.name == "bridge.cjs")) {
                urls.insert(0, a.url.clone());
            }
            let dest = bdir.join("bridge-update.cjs");
            let mut last = String::new();
            for u in urls {
                match download(&u, &dest, 0, |_, _| {}) {
                    Ok(()) => {
                        let meta = serde_json::json!({"connector": rel_v, "script": "bridge/bridge-update.cjs", "source": u});
                        std::fs::write(bdir.join("bridge-update.json"), meta.to_string()).map_err(|e| e.to_string())?;
                        return Ok((dest, rel_v.clone(), format!("อัปเดตตัวเชื่อมเป็น {rel_v} แล้ว (เชื่อมต่อใหม่อัตโนมัติ)")));
                    }
                    Err(e) => last = e,
                }
            }
            if npm_v.is_empty() {
                return Err(format!("ดาวน์โหลดตัวเชื่อมไม่สำเร็จ: {last}"));
            }
        }
        // B) npm install the newest connector next to the config
        if !npm_v.is_empty() && newer(&npm_v, &cur) {
            self.st.lock().unwrap().bridge_msg = format!("กำลังติดตั้ง tiktok-live-connector {npm_v} ด้วย npm...");
            let ndir = self.dir.join("bridge-npm");
            std::fs::create_dir_all(&ndir).map_err(|e| e.to_string())?;
            std::fs::write(ndir.join("package.json"), r#"{"name":"tiktok-shrine-bridge-local","private":true,"type":"module","license":"AGPL-3.0-only"}"#).map_err(|e| e.to_string())?;
            std::fs::write(ndir.join("bridge.js"), BRIDGE_SOURCE).map_err(|e| e.to_string())?;
            let npm = npm_command(node_cmd);
            let mut c = Command::new(&npm);
            c.args(["install", "--no-audit", "--no-fund", "--omit=dev", &format!("tiktok-live-connector@{npm_v}")]).current_dir(&ndir);
            run(&mut c, "npm install").map_err(|e| format!("{e}\n(ต้องติดตั้ง Node.js ที่มี npm ก่อน)"))?;
            let pj = std::fs::read_to_string(ndir.join("node_modules/tiktok-live-connector/package.json")).map_err(|e| e.to_string())?;
            let v = serde_json::from_str::<Value>(&pj).ok().and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from)).unwrap_or(npm_v.clone());
            let meta = serde_json::json!({"connector": v, "script": "bridge-npm/bridge.js", "source": "npm"});
            std::fs::write(bdir.join("bridge-update.json"), meta.to_string()).map_err(|e| e.to_string())?;
            return Ok((ndir.join("bridge.js"), v.clone(), format!("ติดตั้ง tiktok-live-connector {v} จาก npm แล้ว (เชื่อมต่อใหม่อัตโนมัติ)")));
        }
        Err(format!("ตัวเชื่อม TikTok เป็นเวอร์ชันล่าสุดแล้ว ({cur})"))
    }
}

/// npm lives next to node (Windows: npm.cmd)
fn npm_command(node_cmd: &str) -> PathBuf {
    let node = crate::events::find_node(node_cmd);
    let name = if cfg!(windows) { "npm.cmd" } else { "npm" };
    if let Some(d) = node.parent() {
        let p = d.join(name);
        if p.exists() {
            return p;
        }
    }
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions() {
        assert!(newer("v0.2.0", "0.1.1"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(!newer("v0.2.0", "0.2.0"));
        assert!(!newer("0.1.9", "v0.2.0"));
        assert!(newer("2.6.1", "2.5.0"));
    }
    #[test]
    #[cfg(unix)]
    fn rollback_when_new_binary_fails() {
        use std::os::unix::fs::PermissionsExt;
        // install_verified() swaps current_exe(), so the verify step is tested directly here
        let d = std::env::temp_dir().join(format!("tsl-st-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        let bad = d.join("bad");
        std::fs::write(&bad, "#!/bin/sh\necho 'error: GLIBC_2.43 not found' >&2\nexit 127\n").unwrap();
        std::fs::set_permissions(&bad, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(selftest(&bad, std::time::Duration::from_secs(5)).is_err());
        let good = d.join("good");
        std::fs::write(&good, "#!/bin/sh\necho 'selftest ok 9.9.9'\n").unwrap();
        std::fs::set_permissions(&good, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(selftest(&good, std::time::Duration::from_secs(5)).is_ok());
        let hang = d.join("hang");
        std::fs::write(&hang, "#!/bin/sh\nsleep 30\n").unwrap();
        std::fs::set_permissions(&hang, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(selftest(&hang, std::time::Duration::from_secs(1)).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn assets() {
        let a = vec![
            Asset { name: "TikTokShrineLive-Windows-x64.zip".into(), url: "w".into(), size: 1 },
            Asset { name: "TikTokShrineLive-macOS-universal.dmg".into(), url: "m".into(), size: 1 },
            Asset { name: "TikTokShrineLive-macOS-universal.zip".into(), url: "mz".into(), size: 1 },
            Asset { name: "TikTokShrineLive-Linux-x64.tar.gz".into(), url: "l".into(), size: 1 },
            Asset { name: "bridge.cjs".into(), url: "b".into(), size: 1 },
        ];
        let p = pick_asset(&a).unwrap();
        if cfg!(windows) { assert_eq!(p.url, "w") } else if cfg!(target_os = "macos") { assert_eq!(p.url, "mz") } else { assert_eq!(p.url, "l") }
    }
}
