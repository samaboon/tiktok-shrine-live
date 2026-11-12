//! config.toml loading
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub tiktok: TikTokCfg,
    pub gifts: BTreeMap<String, String>,
    pub http: HttpCfg,
    pub window: WindowCfg,
    pub numbers: NumbersCfg,
    pub timing: TimingCfg,
    pub tiers: Vec<TierCfg>,
    pub siamsi: SiamsiCfg,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TikTokCfg {
    pub username: String,
    pub enabled: bool,
    pub node_command: String,
    pub bridge_script: String,
    pub sign_api_key: String,
    pub reconnect_seconds: u64,
    pub default_action: String,
    pub max_plays_per_gift: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HttpCfg {
    pub enabled: bool,
    pub bind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct WindowCfg {
    pub width: f32,
    /// "top" = top part of a 9:16 screen, "full" = full 9:16
    pub layout: String,
    /// height of the overlay in % of a 9:16 screen (layout = "top")
    pub top_percent: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub background: String,
    pub always_on_top: bool,
    pub click_through: bool,
    pub show_hint: bool,
    pub hint_text: String,
    pub show_status: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct NumbersCfg {
    pub digits: Vec<u32>,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TimingCfg {
    pub speed: f32,
    pub reveal_seconds: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TierCfg {
    pub min_diamonds: u64,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SiamsiCfg {
    pub title: String,
    pub fortunes: Vec<String>,
}

impl Default for TikTokCfg {
    fn default() -> Self {
        Self {
            username: String::new(),
            enabled: true,
            node_command: "node".into(),
            bridge_script: "bridge/bridge.js".into(),
            sign_api_key: String::new(),
            reconnect_seconds: 15,
            default_action: "pray".into(),
            max_plays_per_gift: 1,
        }
    }
}
impl Default for HttpCfg {
    fn default() -> Self {
        Self { enabled: true, bind: "127.0.0.1:8787".into() }
    }
}
impl Default for WindowCfg {
    fn default() -> Self {
        Self {
            width: 540.0,
            layout: "top".into(),
            top_percent: 50.0,
            x: None,
            y: None,
            background: "transparent".into(),
            always_on_top: true,
            click_through: false,
            show_hint: true,
            hint_text: String::new(),
            show_status: true,
        }
    }
}
impl Default for NumbersCfg {
    fn default() -> Self {
        Self { digits: vec![2, 3], title: "เลขมงคล".into() }
    }
}
impl Default for TimingCfg {
    fn default() -> Self {
        Self { speed: 1.0, reveal_seconds: 5.0 }
    }
}
impl Default for SiamsiCfg {
    fn default() -> Self {
        Self { title: "เซียมซีใบที่".into(), fortunes: vec![] }
    }
}
impl Default for Config {
    /// Minimal defaults (must not parse DEFAULT_CONFIG: serde(default) would recurse).
    fn default() -> Self {
        Self {
            tiktok: TikTokCfg::default(),
            gifts: BTreeMap::new(),
            http: HttpCfg::default(),
            window: WindowCfg::default(),
            numbers: NumbersCfg::default(),
            timing: TimingCfg::default(),
            tiers: vec![],
            siamsi: SiamsiCfg::default(),
        }
    }
}

impl Config {
    /// The fully documented built-in config (config.toml shipped with the app).
    pub fn builtin() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("built-in default config must parse")
    }
}

impl Config {
    /// Load config; if the file does not exist, write the default one next to it.
    pub fn load(path: &Path) -> Config {
        let mut cfg = match std::fs::read_to_string(path) {
            Ok(s) => match toml::from_str::<Config>(&s) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[config] อ่าน {} ไม่ได้: {e}\n[config] ใช้ค่าเริ่มต้นแทน", path.display());
                    Config::builtin()
                }
            },
            Err(_) => {
                eprintln!("[config] ไม่พบ {} -> สร้างไฟล์ค่าเริ่มต้นให้", path.display());
                if let Some(d) = path.parent() {
                    let _ = std::fs::create_dir_all(d);
                }
                let _ = std::fs::write(path, DEFAULT_CONFIG);
                Config::builtin()
            }
        };
        let def = Config::builtin();
        if cfg.tiers.is_empty() {
            cfg.tiers = def.tiers.clone();
        }
        cfg.tiers.sort_by_key(|t| t.min_diamonds);
        if cfg.siamsi.fortunes.is_empty() {
            cfg.siamsi.fortunes = def.siamsi.fortunes.clone();
        }
        if cfg.numbers.digits.is_empty() {
            cfg.numbers.digits = vec![2, 3];
        }
        if cfg.timing.speed <= 0.05 {
            cfg.timing.speed = 1.0;
        }
        cfg.window.width = cfg.window.width.max(160.0);
        cfg.window.top_percent = cfg.window.top_percent.clamp(20.0, 100.0);
        cfg
    }

    /// Window inner size from width + layout.
    pub fn window_size(&self) -> (f32, f32) {
        let w = self.window.width;
        let full = w * 16.0 / 9.0;
        let h = if self.window.layout == "full" { full } else { full * self.window.top_percent / 100.0 };
        (w.round(), h.round())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let body = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let text = format!("# TikTok Shrine Live - บันทึกจากหน้าตั้งค่าในโปรแกรม\n# (ดูคำอธิบายแต่ละค่าได้ใน config.example.toml)\n\n{body}");
        std::fs::write(path, text).map_err(|e| e.to_string())
    }

    /// Tier index (0-based) for a given total diamond value.
    pub fn tier_for(&self, diamonds: u64) -> usize {
        let mut idx = 0;
        for (i, t) in self.tiers.iter().enumerate() {
            if diamonds >= t.min_diamonds {
                idx = i;
            }
        }
        idx
    }

    /// Lookup gift -> action string ("pray"/"siamsi"/"none").
    pub fn action_for_gift(&self, name: &str, id: u64) -> String {
        let lname = name.trim().to_lowercase();
        for (k, v) in &self.gifts {
            let k = k.trim();
            if (!lname.is_empty() && k.to_lowercase() == lname) || (id != 0 && k == id.to_string()) {
                return v.to_lowercase();
            }
        }
        self.tiktok.default_action.to_lowercase()
    }
}

/// Folder that holds config.toml / assets / bridge: next to the exe if present there, else cwd.
pub fn base_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("assets").exists() || dir.join("config.toml").exists() {
                return dir.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn digit_label(n: u32) -> String {
    const L: [&str; 7] = ["", "หนึ่งตัว", "สองตัว", "สามตัว", "สี่ตัว", "ห้าตัว", "หกตัว"];
    L.get(n as usize).map(|s| s.to_string()).unwrap_or_else(|| format!("{n} ตัว"))
}
