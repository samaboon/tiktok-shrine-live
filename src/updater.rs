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
    // bundled Node runtime (Windows / Linux folder installs; on macOS it ships inside the .app)
    pub runtime_current: Option<String>,
    pub runtime_release: Option<String>,
    pub runtime_managed: bool,
    /// runtime replaced -> app reconnects the bridge
    pub bridge_reconnect: bool,
}

/// Platform runtime package in a release ("runtime-node-win-x64.zip" ...).
pub fn runtime_asset_name() -> Option<&'static str> {
    if cfg!(windows) {
        Some("runtime-node-win-x64.zip")
    } else if cfg!(target_os = "linux") {
        Some("runtime-node-x64-glibc.tar.gz")
    } else {
        None
    }
}

impl UpState {
    /// What the single "มีอัปเดต" button would do, in install order (runtime, connector, app).
    pub fn pending(&self) -> Vec<&'static str> {
        let mut v = vec![];
        if self.runtime_needed() {
            v.push("runtime");
        }
        if self.bridge_release.as_ref().map(|r| newer(r, &self.bridge_current)).unwrap_or(false) {
            v.push("connector");
        }
        if self.available {
            v.push("app");
        }
        v
    }
    fn runtime_needed(&self) -> bool {
        let has_asset = self.latest.as_ref().map(|l| l.assets.iter().any(|a| Some(a.name.as_str()) == runtime_asset_name())).unwrap_or(false);
        match (&self.runtime_release, &self.runtime_current) {
            (Some(r), Some(c)) => self.runtime_managed && has_asset && newer(r, c),
            (Some(_), None) => self.runtime_managed && has_asset,
            _ => false,
        }
    }
    /// Thai summary for the button / badge, e.g. "แอป v0.2.2 · ตัวเชื่อม 2.1.1 · ตัวรัน v22.24.0"
    pub fn pending_text(&self) -> String {
        self.pending()
            .iter()
            .map(|k| match *k {
                "runtime" => format!("ตัวรัน {}", self.runtime_release.clone().unwrap_or_default()),
                "connector" => format!("ตัวเชื่อม {}", self.bridge_release.clone().unwrap_or_default()),
                _ => format!("แอป {}", self.latest.as_ref().map(|l| l.tag.clone()).unwrap_or_default()),
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }
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
    // runtime packages are separate assets, never the app itself
    let find = |f: &dyn Fn(&str) -> bool| assets.iter().filter(|a| !low(a).starts_with("runtime")).find(|a| f(&low(a))).cloned();
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
    // the app package also carries runtime/ : install it when this folder has none or an older one
    if let Some(rt) = find_file(&ex, &|p| p.is_dir() && p.file_name().map(|n| n == "runtime").unwrap_or(false), 2) {
        if let Some(dest) = crate::runtime::runtime_dir() {
            let newv = std::fs::read_to_string(rt.join("NODE_VERSION")).unwrap_or_default().trim().to_string();
            let curv = std::fs::read_to_string(dest.join("NODE_VERSION")).unwrap_or_default().trim().to_string();
            if !dest.join(crate::runtime::NODE_EXE).exists() || (!newv.is_empty() && newer(&newv, &curv)) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(rt.join(crate::runtime::NODE_EXE), std::fs::Permissions::from_mode(0o755));
                }
                if let Err(e) = swap_dir(&rt, &dest, &|d| crate::runtime::node_version_of(&d.join(crate::runtime::NODE_EXE)).map(|_| ())) {
                    eprintln!("[update] ตัวรัน: {e}");
                }
            }
        }
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

/// Replace directory `dest` with a copy of `src`; `verify` runs on the installed copy, failure restores the old one.
/// The old directory is renamed (not deleted) first because a running node.exe can be renamed but not deleted on Windows.
pub fn swap_dir(src: &Path, dest: &Path, verify: &dyn Fn(&Path) -> Result<(), String>) -> Result<(), String> {
    let newd = dest.with_file_name("runtime.new");
    let old = dest.with_file_name("runtime.old");
    let _ = std::fs::remove_dir_all(&newd);
    copy_dir(src, &newd).map_err(|e| format!("คัดลอกตัวรันไม่ได้: {e}"))?;
    let _ = std::fs::remove_dir_all(&old);
    let had_old = dest.exists();
    if had_old {
        std::fs::rename(dest, &old).map_err(|e| format!("ย้ายตัวรันเดิมไม่ได้: {e}"))?;
    }
    if let Err(e) = std::fs::rename(&newd, dest) {
        if had_old {
            let _ = std::fs::rename(&old, dest);
        }
        return Err(format!("ติดตั้งตัวรันไม่ได้: {e}"));
    }
    if let Err(e) = verify(dest) {
        let _ = std::fs::remove_dir_all(dest);
        if had_old {
            let _ = std::fs::rename(&old, dest);
        }
        return Err(format!("{e} — ย้อนกลับเป็นตัวรันเดิมแล้ว"));
    }
    let _ = std::fs::remove_dir_all(&old); // may fail while the old node.exe still runs; cleaned at next start
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for e in std::fs::read_dir(src)? {
        let e = e?;
        let p = e.path();
        let t = dst.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &t)?;
        } else {
            std::fs::copy(&p, &t)?;
        }
    }
    Ok(())
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
        let st = UpState { current, bridge_current, runtime_current: crate::runtime::bundled_version(), runtime_managed: crate::runtime::runtime_dir().is_some(), ..Default::default() };
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
            "runtime_current": s.runtime_current, "runtime_release": s.runtime_release, "runtime_managed": s.runtime_managed,
            "pending": s.pending(), "pending_text": s.pending_text(),
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
            if let Ok(r) = &res {
                me.check_runtime(r);
            }
            me.check_bridge(res.ok());
        });
    }

    fn check_runtime(&self, rel: &ReleaseInfo) {
        let v = rel.assets.iter().find(|a| a.name == "runtime-version.json").and_then(|a| fetch_text(&a.url).ok()).and_then(|t| serde_json::from_str::<Value>(&t).ok()).and_then(|v| v.get("node").and_then(|s| s.as_str()).map(String::from));
        let mut s = self.st.lock().unwrap();
        s.runtime_current = crate::runtime::bundled_version();
        s.runtime_release = v;
        eprintln!("[update] ตัวรัน Node: ใช้อยู่ {:?} | release {:?}", s.runtime_current, s.runtime_release);
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

    /// Download + install the app update for this OS (single item; the main UI uses `update_all`).
    pub fn apply(&self) {
        let (rel, busy) = {
            let s = self.st.lock().unwrap();
            (s.latest.clone(), s.busy)
        };
        let Some(rel) = rel else { return };
        if busy {
            return;
        }
        self.st.lock().unwrap().busy = true;
        let me = self.clone();
        std::thread::spawn(move || me.finish_app(&rel, ""));
    }

    fn set_phase(&self, p: String) {
        eprintln!("[update] {p}");
        let mut s = self.st.lock().unwrap();
        s.phase = p;
        s.progress = None;
    }
    fn fail(&self, e: String) {
        eprintln!("[update] ผิดพลาด: {e}");
        let mut s = self.st.lock().unwrap();
        s.error = e;
        s.busy = false;
        s.progress = None;
    }

    /// Download + install + verify the app; on success sets `restart`. `step` prefixes progress text.
    fn finish_app(&self, rel: &ReleaseInfo, step: &str) {
        let Some(asset) = pick_asset(&rel.assets) else {
            self.fail(format!("ยังไม่มีไฟล์สำหรับเครื่องนี้ใน {} - ดาวน์โหลดเองที่ {}", rel.tag, rel.html_url));
            return;
        };
        let work = self.dir.join("update-tmp");
        let _ = std::fs::remove_dir_all(&work);
        let _ = std::fs::create_dir_all(&work);
        let file = work.join(&asset.name);
        self.set_phase(format!("{step}กำลังดาวน์โหลดแอป {} ({})", rel.tag, asset.name));
        let st = self.st.clone();
        if let Err(e) = download(&asset.url, &file, asset.size, |a, b| st.lock().unwrap().progress = Some((a, b))) {
            let _ = std::fs::remove_dir_all(&work);
            self.fail(e);
            return;
        }
        self.set_phase(format!("{step}กำลังติดตั้งแอปและทดสอบเปิด..."));
        match install(&file, &work) {
            Ok(exe) => {
                let _ = std::fs::remove_dir_all(&work);
                let mut s = self.st.lock().unwrap();
                s.phase = format!("ติดตั้ง {} แล้ว กำลังเปิดใหม่...", rel.tag);
                s.restart = Some(exe);
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&work);
                self.fail(format!("ติดตั้งไม่สำเร็จ: {e}"));
            }
        }
    }

    /// The ONE update button: runtime -> connector -> app (restart last). Each step verifies and rolls back on failure.
    pub fn update_all(&self, node_cmd: String) {
        {
            let mut s = self.st.lock().unwrap();
            if s.busy || s.pending().is_empty() {
                return;
            }
            s.busy = true;
            s.error.clear();
        }
        let me = self.clone();
        std::thread::spawn(move || {
            let snap = me.snapshot();
            let items = snap.pending();
            let n = items.len();
            let mut done_msgs = vec![];
            for (i, item) in items.iter().enumerate() {
                let step = if n > 1 { format!("({}/{}) ", i + 1, n) } else { String::new() };
                match *item {
                    "runtime" => match me.install_runtime(&snap, &step) {
                        Ok(v) => {
                            let mut s = me.st.lock().unwrap();
                            s.runtime_current = Some(v.clone());
                            s.bridge_reconnect = true;
                            done_msgs.push(format!("ตัวรัน {v}"));
                        }
                        Err(e) => return me.fail(format!("อัปเดตตัวรันไม่สำเร็จ (ใช้ตัวเดิมต่อ): {e}")),
                    },
                    "connector" => {
                        me.set_phase(format!("{step}กำลังอัปเดตตัวเชื่อม TikTok {}...", snap.bridge_release.clone().unwrap_or_default()));
                        match me.update_bridge_release(&node_cmd) {
                            Ok((script, ver, _)) => {
                                let mut s = me.st.lock().unwrap();
                                s.bridge_current = ver.clone();
                                s.bridge_new_script = Some(script);
                                done_msgs.push(format!("ตัวเชื่อม {ver}"));
                            }
                            Err(e) => return me.fail(format!("อัปเดตตัวเชื่อมไม่สำเร็จ (ใช้ตัวเดิมต่อ): {e}")),
                        }
                    }
                    _ => {
                        if let Some(rel) = snap.latest.clone() {
                            me.finish_app(&rel, &step);
                        }
                        return; // finish_app sets restart or error
                    }
                }
            }
            let mut s = me.st.lock().unwrap();
            s.busy = false;
            s.progress = None;
            s.phase = format!("อัปเดตแล้ว: {}", done_msgs.join(" · "));
            eprintln!("[update] {}", s.phase);
        });
    }

    /// Replace <app>/runtime with the release's runtime package; verify `node --version`; roll back on failure.
    fn install_runtime(&self, snap: &UpState, step: &str) -> Result<String, String> {
        let name = runtime_asset_name().ok_or("ระบบนี้อัปเดตตัวรันพร้อมแอป")?;
        let rel = snap.latest.as_ref().ok_or("ไม่มีข้อมูล release")?;
        let asset = rel.assets.iter().find(|a| a.name == name).ok_or("ไม่มีไฟล์ตัวรันใน release")?;
        let want = snap.runtime_release.clone().unwrap_or_default();
        let dest = crate::runtime::runtime_dir().ok_or("ไม่พบโฟลเดอร์โปรแกรม")?;
        let work = self.dir.join("update-rt");
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
        let file = work.join(name);
        self.set_phase(format!("{step}กำลังดาวน์โหลดตัวรัน {want}"));
        let st = self.st.clone();
        let r = (|| {
            download(&asset.url, &file, asset.size, |a, b| st.lock().unwrap().progress = Some((a, b)))?;
            self.set_phase(format!("{step}กำลังติดตั้งตัวรัน {want}..."));
            let ex = work.join("x");
            extract(&file, &ex)?;
            let node = find_file(&ex, &|p| p.is_file() && p.file_name().map(|n| n == crate::runtime::NODE_EXE).unwrap_or(false), 3).ok_or("ไม่พบ node ในไฟล์ตัวรัน")?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o755));
            }
            let got = crate::runtime::node_version_of(&node)?;
            if !want.is_empty() && got != want {
                return Err(format!("เวอร์ชันไม่ตรง ({got} ≠ {want})"));
            }
            let src_dir = node.parent().unwrap().to_path_buf();
            swap_dir(&src_dir, &dest, &|d| crate::runtime::node_version_of(&d.join(crate::runtime::NODE_EXE)).map(|_| ()))?;
            Ok(got)
        })();
        let _ = std::fs::remove_dir_all(&work);
        r
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

    /// Connector update from the prebuilt release bundle only (used by the one-button update), verified with `node --check`.
    fn update_bridge_release(&self, node_cmd: &str) -> Result<(PathBuf, String, String), String> {
        let s = self.snapshot();
        let rel_v = s.bridge_release.clone().ok_or("ไม่มีข้อมูลตัวเชื่อม")?;
        let bdir = self.dir.join("bridge");
        std::fs::create_dir_all(&bdir).map_err(|e| e.to_string())?;
        let mut urls = vec![format!("{}/{}/releases/download/{}/bridge.cjs", self.dl_base, self.repo, self.bridge_tag)];
        if let Some(a) = s.latest.as_ref().and_then(|r| r.assets.iter().find(|a| a.name == "bridge.cjs")) {
            urls.insert(0, a.url.clone());
        }
        let tmp = bdir.join("bridge-download.cjs");
        let mut last = String::new();
        for u in urls {
            match download(&u, &tmp, 0, |_, _| {}) {
                Ok(()) => {
                    // verify the new bundle parses with the runtime that will run it; keep the old one otherwise
                    let (node, _) = crate::runtime::resolve_node(node_cmd);
                    let mut c = Command::new(&node);
                    c.arg("--check").arg(&tmp);
                    if let Err(e) = run(&mut c, "ตรวจตัวเชื่อม") {
                        let _ = std::fs::remove_file(&tmp);
                        return Err(e);
                    }
                    let dest = bdir.join("bridge-update.cjs");
                    std::fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
                    let meta = serde_json::json!({"connector": rel_v, "script": "bridge/bridge-update.cjs", "source": u});
                    std::fs::write(bdir.join("bridge-update.json"), meta.to_string()).map_err(|e| e.to_string())?;
                    return Ok((dest, rel_v.clone(), format!("อัปเดตตัวเชื่อมเป็น {rel_v} แล้ว")));
                }
                Err(e) => last = e,
            }
        }
        Err(format!("ดาวน์โหลดตัวเชื่อมไม่สำเร็จ: {last}"))
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
    fn swap_dir_installs_and_rolls_back() {
        let base = std::env::temp_dir().join(format!("tsl-swap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (dest, good, bad) = (base.join("runtime"), base.join("good"), base.join("bad"));
        for (d, v) in [(&dest, "old"), (&good, "new"), (&bad, "broken")] {
            std::fs::create_dir_all(d).unwrap();
            std::fs::write(d.join("NODE_VERSION"), v).unwrap();
        }
        let ok = |d: &Path| if std::fs::read_to_string(d.join("NODE_VERSION")).unwrap() == "broken" { Err("bad".to_string()) } else { Ok(()) };
        assert!(swap_dir(&bad, &dest, &ok).is_err());
        assert_eq!(std::fs::read_to_string(dest.join("NODE_VERSION")).unwrap(), "old", "rolled back");
        swap_dir(&good, &dest, &ok).unwrap();
        assert_eq!(std::fs::read_to_string(dest.join("NODE_VERSION")).unwrap(), "new");
        assert!(!base.join("runtime.new").exists());
        let _ = std::fs::remove_dir_all(&base);
    }
    #[test]
    fn one_button_pending_order() {
        let mut s = UpState { current: "0.2.0".into(), bridge_current: "2.1.0".into(), runtime_managed: true, ..Default::default() };
        assert!(s.pending().is_empty());
        let mut assets = vec![Asset { name: "bridge.cjs".into(), url: "b".into(), size: 1 }];
        if let Some(n) = runtime_asset_name() {
            assets.push(Asset { name: n.into(), url: "r".into(), size: 1 });
        }
        s.latest = Some(ReleaseInfo { tag: "v0.2.1".into(), assets, ..Default::default() });
        s.available = true;
        s.bridge_release = Some("2.1.1".into());
        s.runtime_release = Some("v22.23.5".into());
        s.runtime_current = None; // updated from 0.2.0 -> no bundled runtime yet
        let want: Vec<&str> = if runtime_asset_name().is_some() { vec!["runtime", "connector", "app"] } else { vec!["connector", "app"] };
        assert_eq!(s.pending(), want);
        s.runtime_current = Some("v22.23.5".into());
        s.bridge_current = "2.1.1".into();
        assert_eq!(s.pending(), vec!["app"]);
        assert!(s.pending_text().contains("v0.2.1"));
    }
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
