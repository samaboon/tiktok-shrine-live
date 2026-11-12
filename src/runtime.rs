//! Bundled Node.js runtime for the TikTok connector (bridge.cjs).
//! Users never install Node.js: release packages ship the official portable `node` binary
//!   Windows zip / Linux tar.gz : <app folder>/runtime/node(.exe)
//!   macOS .app                 : Contents/MacOS/node   (inside the signed bundle)
//! Resolution order: bundled runtime -> system Node (PATH / usual install dirs) -> config `node_command`.
//! A custom `node_command` (advanced setting) is only used when it points to an existing file
//! or when nothing else was found.
use std::path::{Path, PathBuf};

pub const NODE_EXE: &str = if cfg!(windows) { "node.exe" } else { "node" };

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.canonicalize().ok().or(Some(p))).and_then(|p| p.parent().map(Path::to_path_buf))
}

/// Folder the updater may (re)write the runtime into (not inside a macOS bundle).
pub fn runtime_dir() -> Option<PathBuf> {
    if let Ok(d) = std::env::var("TSL_RUNTIME_DIR") {
        return Some(PathBuf::from(d));
    }
    if cfg!(target_os = "macos") {
        return None;
    }
    exe_dir().map(|d| d.join("runtime"))
}

/// Path of the bundled node binary, if present.
pub fn bundled_node() -> Option<PathBuf> {
    let mut c = vec![];
    if let Some(r) = runtime_dir() {
        c.push(r.join(NODE_EXE));
    }
    if let Some(d) = exe_dir() {
        c.push(d.join("runtime").join(NODE_EXE));
        if cfg!(target_os = "macos") {
            c.push(d.join("node")); // Contents/MacOS/node
            c.push(d.join("../Resources/runtime/node"));
        }
    }
    let p = c.into_iter().find(|p| p.is_file())?;
    #[cfg(target_os = "macos")]
    {
        // the bundle was already approved by the user; drop a leftover quarantine flag on the nested binary (once)
        static ONCE: std::sync::Once = std::sync::Once::new();
        let q = p.clone();
        ONCE.call_once(move || {
            let _ = std::process::Command::new("xattr").args(["-d", "com.apple.quarantine"]).arg(&q).stderr(std::process::Stdio::null()).status();
        });
    }
    Some(p)
}

/// Version of the bundled runtime ("v22.23.5"), from runtime/NODE_VERSION.
pub fn bundled_version() -> Option<String> {
    let node = bundled_node()?;
    let d = node.parent()?;
    for f in [d.join("NODE_VERSION"), d.join("../Resources/runtime/NODE_VERSION"), d.join("../Resources/NODE_VERSION")] {
        if let Ok(s) = std::fs::read_to_string(f) {
            let s = s.trim().to_string();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    Some("?".into())
}

fn is_auto(cmd: &str) -> bool {
    let c = cmd.trim();
    c.is_empty() || c == "node" || c == "auto"
}

/// Which node to run the connector with, and where it came from ("bundled" / "system" / "custom" / "none").
pub fn resolve_node(cmd: &str) -> (PathBuf, &'static str) {
    if !is_auto(cmd) && Path::new(cmd.trim()).is_file() {
        return (PathBuf::from(cmd.trim()), "custom");
    }
    if let Some(p) = bundled_node() {
        return (p, "bundled");
    }
    let sys = crate::events::find_node("node");
    if sys.is_file() {
        return (sys, "system");
    }
    if !is_auto(cmd) {
        return (PathBuf::from(cmd.trim()), "custom");
    }
    (PathBuf::from("node"), "none")
}

/// Thai label for the settings UI.
pub fn describe(cmd: &str) -> String {
    let (p, src) = resolve_node(cmd);
    match src {
        "bundled" => format!("ใช้ตัวที่มากับโปรแกรม ({})", bundled_version().unwrap_or_default()),
        "system" => format!("ใช้ Node.js ในเครื่อง ({})", p.display()),
        "custom" => format!("ใช้ตามที่ตั้งเอง ({})", p.display()),
        _ => "ไม่พบตัวรันตัวเชื่อม — ดาวน์โหลดโปรแกรมใหม่ หรือกดอัปเดต".into(),
    }
}

/// Remove a runtime left over from a previous swap (it may have been in use then).
pub fn cleanup_old() {
    if let Some(r) = runtime_dir() {
        let _ = std::fs::remove_dir_all(r.with_file_name("runtime.old"));
    }
}

/// `node --version` of a binary (with timeout), used to verify a runtime before/after swapping.
pub fn node_version_of(node: &Path) -> Result<String, String> {
    let mut c = std::process::Command::new(node);
    c.arg("--version").stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000);
    }
    let mut ch = c.spawn().map_err(|e| format!("เปิด {} ไม่ได้: {e}", node.display()))?;
    let t0 = std::time::Instant::now();
    loop {
        match ch.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if t0.elapsed() > std::time::Duration::from_secs(20) => {
                let _ = ch.kill();
                return Err("node ไม่ตอบสนอง".into());
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let out = ch.wait_with_output().map_err(|e| e.to_string())?;
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if out.status.success() && v.starts_with('v') { Ok(v) } else { Err(format!("node ใช้งานไม่ได้ ({}) {}", out.status, String::from_utf8_lossy(&out.stderr).trim())) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_values() {
        assert!(is_auto(""));
        assert!(is_auto("node"));
        assert!(is_auto(" auto "));
        assert!(!is_auto("/opt/node/bin/node"));
    }
}
