// No console window on Windows release builds (double-click app).
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod config;
mod events;
mod krathong;
mod scene;
mod updater;

use app::{AppInit, ShotPlan, ShrineApp};
use config::Config;
use events::{Action, BridgeState, EventSink, EventSource, HttpSource, TikTokBridgeSource};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

const FONT_REG: &[u8] = include_bytes!("../assets/fonts/NotoSansThai-Regular.ttf");
const FONT_BOLD: &[u8] = include_bytes!("../assets/fonts/NotoSansThai-Bold.ttf");
const LATIN_REG: &[u8] = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
const LATIN_BOLD: &[u8] = include_bytes!("../assets/fonts/NotoSans-Bold.ttf");
const ICON_PNG: &[u8] = include_bytes!("../assets/icon/icon-256.png");
/// Single-file bundle of bridge/bridge.js + tiktok-live-connector (built with esbuild).
const BRIDGE_JS: &[u8] = include_bytes!("../bridge/dist/bridge.cjs");

const APP_DIR_NAME: &str = "TikTokShrineLive";

struct Args {
    config: Option<PathBuf>,
    demo: Vec<(Action, usize, String)>,
    shot_at: Option<f32>,
    out: Option<PathBuf>,
    settings_tab: Option<usize>,
    bg: Option<String>,
    layout: Option<String>,
    width: Option<f32>,
    seed: Option<u64>,
    preview: bool,
    settings: bool,
    no_status: bool,
    stopped: bool,
    theme: Option<String>,
    record: Option<(PathBuf, f32, f32, u32)>,
    http_in_shot: bool,
    fake_version: Option<String>,
}

fn parse_args() -> Args {
    let mut a = Args { config: None, demo: vec![], shot_at: None, out: None, settings_tab: None, bg: None, layout: None, width: None, seed: None, preview: false, settings: false, no_status: false, stopped: false, theme: None, record: None, http_in_shot: false, fake_version: None };
    let mut it = std::env::args().skip(1);
    while let Some(x) = it.next() {
        match x.as_str() {
            "--config" => a.config = it.next().map(PathBuf::from),
            // pretend to be an older version (to test the updater)
            "--fake-version" => a.fake_version = it.next(),
            // --demo pray:4:ชื่อ   (action:tier(1-5):name)
            "--demo" => {
                if let Some(v) = it.next() {
                    let mut p = v.splitn(3, ':');
                    let act = p.next().and_then(Action::parse).unwrap_or(Action::Pray);
                    let tier = p.next().and_then(|t| t.parse::<usize>().ok()).unwrap_or(1).max(1) - 1;
                    let name = p.next().unwrap_or("สมชาย").to_string();
                    a.demo.push((act, tier, name));
                }
            }
            "--shot-at" => a.shot_at = it.next().and_then(|v| v.parse().ok()),
            "--out" => a.out = it.next().map(PathBuf::from),
            "--settings-tab" => {
                a.settings_tab = it.next().and_then(|v| v.parse().ok());
                a.settings = true;
            }
            "--bg" => a.bg = it.next(),
            "--layout" => a.layout = it.next(),
            "--width" => a.width = it.next().and_then(|v| v.parse().ok()),
            "--seed" => a.seed = it.next().and_then(|v| v.parse().ok()),
            "--preview" => a.preview = true,
            "--settings" => a.settings = true,
            "--no-status" => a.no_status = true,
            "--stopped" => a.stopped = true,
            "--theme" | "--mode" => a.theme = it.next(),
            "--http" => a.http_in_shot = true,
            // --record DIR START END FPS : dump frames (PNG) for GIF/MP4 previews
            "--record" => {
                let d = it.next().map(PathBuf::from);
                let s0 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let s1 = it.next().and_then(|v| v.parse().ok()).unwrap_or(10.0);
                let fps = it.next().and_then(|v| v.parse().ok()).unwrap_or(20);
                if let Some(d) = d {
                    a.record = Some((d, s0, s1, fps));
                }
            }
            "--help" | "-h" => {
                println!("tiktok-shrine-live [--config PATH] [--settings]\n  test: --demo pray:5:ชื่อ --shot-at 7 --out shot.png [--bg transparent|green|scene] [--layout top|full] [--preview]");
                std::process::exit(0);
            }
            _ => {}
        }
    }
    a
}

fn user_config_dir() -> PathBuf {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"));
    #[cfg(not(any(windows, target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));
    base.unwrap_or_else(|| PathBuf::from(".")).join(APP_DIR_NAME)
}

fn writable(dir: &Path) -> bool {
    let probe = dir.join(".write_test");
    let ok = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(probe);
    ok
}

/// Where config.toml lives:
///  1. ./config.toml (current dir, for development)
///  2. next to the exe (portable zip on Windows)
///  3. per-user dir (macOS .app, read-only installs)
fn resolve_config_dir() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.join("config.toml").exists() {
            return cwd;
        }
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        let in_bundle = exe_dir.to_string_lossy().contains(".app/Contents");
        if exe_dir.join("config.toml").exists() || (!in_bundle && writable(&exe_dir)) {
            return exe_dir;
        }
    }
    let d = user_config_dir();
    let _ = std::fs::create_dir_all(&d);
    d
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("thai".into(), Arc::new(egui::FontData::from_static(FONT_REG)));
    fonts.font_data.insert("thai_bold".into(), Arc::new(egui::FontData::from_static(FONT_BOLD)));
    fonts.font_data.insert("latin".into(), Arc::new(egui::FontData::from_static(LATIN_REG)));
    fonts.font_data.insert("latin_bold".into(), Arc::new(egui::FontData::from_static(LATIN_BOLD)));
    let prop = fonts.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    // Noto Sans (Latin/digits) + Noto Sans Thai, then egui defaults (emoji/symbols) as fallback
    let mut p = vec!["latin".to_string(), "thai".to_string()];
    p.extend(prop.iter().cloned());
    fonts.families.insert(egui::FontFamily::Proportional, p);
    let mut b = vec!["latin_bold".to_string(), "thai_bold".to_string()];
    b.extend(prop.iter().cloned());
    fonts.families.insert(egui::FontFamily::Name("bold".into()), b);
    if let Some(m) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        m.push("thai".into());
    }
    ctx.set_fonts(fonts);
}

fn icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(ICON_PNG).ok()?.to_rgba8();
    Some(egui::IconData { width: img.width(), height: img.height(), rgba: img.into_raw() })
}

/// Write the embedded bridge next to the config so only Node.js is needed.
fn extract_bridge(dir: &Path) -> PathBuf {
    let p = dir.join("bridge").join("bridge.cjs");
    let same = std::fs::read(&p).map(|b| b == BRIDGE_JS).unwrap_or(false);
    if !same {
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        if let Err(e) = std::fs::write(&p, BRIDGE_JS) {
            eprintln!("[bridge] เขียน {} ไม่ได้: {e}", p.display());
        }
    }
    p
}

fn main() -> eframe::Result<()> {
    let args = parse_args();
    let dir = resolve_config_dir();
    let cfg_path = args.config.clone().unwrap_or_else(|| dir.join("config.toml"));
    let mut cfg = Config::load(&cfg_path);
    let shot_mode = args.shot_at.is_some();
    if let Some(b) = &args.bg {
        cfg.window.background = b.clone();
    }
    if let Some(l) = &args.layout {
        cfg.window.layout = l.clone();
    }
    if let Some(w) = args.width {
        cfg.window.width = w;
    }
    if args.no_status {
        cfg.window.show_status = false;
    }
    eprintln!("[config] {}", cfg_path.display());

    let assets_dir = [dir.join("assets"), config::base_dir().join("assets")].into_iter().find(|p| p.exists()).unwrap_or_else(|| dir.join("assets"));

    updater::cleanup_old();
    let mut bridge_ver = updater::builtin_bridge_version();
    let script = if !cfg.tiktok.bridge_script.trim().is_empty() && dir.join(&cfg.tiktok.bridge_script).is_file() && cfg.tiktok.bridge_script.trim() != "bridge/bridge.js" {
        bridge_ver = format!("{} (กำหนดเอง)", cfg.tiktok.bridge_script);
        dir.join(&cfg.tiktok.bridge_script)
    } else if let Some((p, v)) = updater::bridge_override(&dir) {
        eprintln!("[bridge] ใช้ตัวเชื่อมที่อัปเดตแล้ว {v}: {}", p.display());
        bridge_ver = v;
        p
    } else {
        extract_bridge(&dir)
    };
    let current_ver = args.fake_version.clone().unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let upd = updater::Updater::new(&cfg.update.repo, &cfg.update.api_base, &cfg.update.download_base, &cfg.update.bridge_tag, dir.clone(), current_ver, bridge_ver);
    if cfg.update.check_on_start && !shot_mode {
        upd.check();
    }
    let bridge = Arc::new(Mutex::new(BridgeState {
        username: cfg.tiktok.username.trim().trim_start_matches('@').to_string(),
        sign_key: cfg.tiktok.sign_api_key.clone(),
        node: cfg.tiktok.node_command.clone(),
        script,
        enabled: cfg.tiktok.enabled && !shot_mode,
        reconnect: Duration::from_secs(cfg.tiktok.reconnect_seconds.max(3)),
        generation: 0,
        child: None,
        status: String::new(),
    }));

    let (w, h) = cfg.window_size();
    let mut vb = egui::ViewportBuilder::default()
        .with_title("TikTok Shrine Live")
        .with_app_id("tiktok-shrine-live")
        .with_inner_size([w, h])
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_mouse_passthrough(cfg.window.click_through && !shot_mode);
    if cfg.window.always_on_top {
        vb = vb.with_always_on_top();
    }
    if let (Some(x), Some(y)) = (cfg.window.x, cfg.window.y) {
        vb = vb.with_position([x, y]);
    }
    if let Some(i) = icon() {
        vb = vb.with_icon(i);
    }
    let options = eframe::NativeOptions { viewport: vb, renderer: eframe::Renderer::Glow, ..Default::default() };

    let shot = args.shot_at.map(|at| ShotPlan {
        at,
        out: args.out.clone().unwrap_or_else(|| PathBuf::from("shot.png")),
        settings_tab: args.settings_tab,
        preview_bg: args.preview,
        requested: false,
        done: Arc::new(AtomicBool::new(false)),
        frames_after: 0,
    });
    let open_settings = args.settings || (!shot_mode && cfg.tiktok.username.trim().is_empty());

    let http_cfg = cfg.http.clone();
    let state = Arc::new(Mutex::new(String::from("{}")));
    let state_app = state.clone();
    let http_in_shot = args.http_in_shot;
    let bridge2 = bridge.clone();
    eframe::run_native(
        "TikTok Shrine Live",
        options,
        Box::new(move |cc| {
            setup_fonts(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            cc.egui_ctx.all_styles_mut(|st| {
                use egui::{FontFamily::Proportional, FontId, TextStyle};
                st.text_styles.insert(TextStyle::Body, FontId::new(16.0, Proportional));
                st.text_styles.insert(TextStyle::Button, FontId::new(16.0, Proportional));
                st.text_styles.insert(TextStyle::Heading, FontId::new(22.0, egui::FontFamily::Name("bold".into())));
                st.text_styles.insert(TextStyle::Small, FontId::new(13.0, Proportional));
                st.text_styles.insert(TextStyle::Monospace, FontId::new(14.0, egui::FontFamily::Monospace));
                st.spacing.button_padding = egui::vec2(10.0, 5.0);
            });
            let (tx, rx) = mpsc::channel();
            let sink = EventSink::new(tx, cc.egui_ctx.clone());
            let mut sources: Vec<Box<dyn EventSource>> = vec![];
            if http_cfg.enabled && (!shot_mode || http_in_shot) {
                sources.push(Box::new(HttpSource { bind: http_cfg.bind.clone(), state: state.clone() }));
            }
            if !shot_mode {
                sources.push(Box::new(TikTokBridgeSource { ctl: bridge2.clone() }));
            }
            for s in sources {
                eprintln!("[source] {}", s.name());
                s.start(sink.clone());
            }
            Ok(Box::new(ShrineApp::new(AppInit { cfg, cfg_path, rx, bridge: bridge2, assets_dir, shot, demo: args.demo, seed: args.seed, open_settings, start_stopped: args.stopped, state: state_app, theme: args.theme.clone(), record: args.record.clone(), updater: upd.clone() })))
        }),
    )
}
