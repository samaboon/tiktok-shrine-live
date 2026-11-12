//! config.toml loading
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const DEFAULT_CONFIG: &str = include_str!("../config.toml");

/// Official TikTok *web* recharge package quoted by KTC: 30 coins = 11 THB.
/// App-store prices are higher (same article: iOS 21 coins = 15 THB).
/// Source last modified 2026-06-04 (JSON-LD); we could not load tiktok.com/coin
/// from this environment (HTTP 403 / captcha).
pub const DEFAULT_THB_PER_COIN: f64 = 11.0 / 30.0;
pub const DEFAULT_THB_RATE_URL: &str = "https://www.ktc.co.th/article/knowledge/tips/top-up-tiktok-coins-via-credit-card";
pub const DEFAULT_THB_RATE_DATE: &str = "2026-06-04";
pub const DEFAULT_THB_RATE_SOURCE: &str = "แพ็กเกจเว็บทางการ TikTok 30 เหรียญ = 11 บาท ตามบทความ KTC — ราคาในแอปสูงกว่า (เช่น iOS 21 เหรียญ = 15 บาท)";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub tiktok: TikTokCfg,
    pub gifts: GiftsCfg,
    pub http: HttpCfg,
    pub window: WindowCfg,
    pub numbers: NumbersCfg,
    pub timing: TimingCfg,
    pub tiers: Vec<TierCfg>,
    pub siamsi: SiamsiCfg,
    pub theme: ThemeCfg,
    pub krathong: KrathongCfg,
    pub cat: CatCfg,
    pub dance: DanceCfg,
    pub dunk: DunkCfg,
    pub update: UpdateCfg,
}

/// Gift-action map plus an optional baht-per-coin rate for the picker / docs.
/// `thb_per_coin` lives in the same `[gifts]` table as the mappings
/// (`[gifts] thb_per_coin = 0.3666…`). Set it to 0 to hide estimated THB prices.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GiftsCfg {
    /// Baht a Thai viewer pays per coin (diamond) on the official web recharge.
    /// `0` = do not show a THB estimate.
    pub thb_per_coin: f64,
    /// Short citation shown in the settings UI (source + package used).
    pub thb_rate_source: String,
    /// ISO date (`YYYY-MM-DD`) of the cited source.
    pub thb_rate_date: String,
    /// gift name or numeric id -> "pray" | "siamsi" | "none"
    #[serde(flatten)]
    pub map: BTreeMap<String, String>,
}
impl Default for GiftsCfg {
    fn default() -> Self {
        Self {
            thb_per_coin: DEFAULT_THB_PER_COIN,
            thb_rate_source: DEFAULT_THB_RATE_SOURCE.into(),
            thb_rate_date: DEFAULT_THB_RATE_DATE.into(),
            map: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct UpdateCfg {
    /// check GitHub Releases when the app starts
    pub check_on_start: bool,
    pub repo: String,
    pub api_base: String,
    pub download_base: String,
    /// rolling release tag that carries the newest TikTok bridge bundle
    pub bridge_tag: String,
}
impl Default for UpdateCfg {
    fn default() -> Self {
        Self { check_on_start: true, repo: "samaboon/tiktok-shrine-live".into(), api_base: "https://api.github.com".into(), download_base: "https://github.com".into(), bridge_tag: "bridge-latest".into() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ThemeCfg {
    /// "shrine" / "krathong" / "cat" / "dance" / "dunk"
    pub mode: String,
}
impl Default for ThemeCfg {
    fn default() -> Self {
        Self { mode: "shrine".into() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct KrathongCfg {
    pub hint_text: String,
    pub wish_title: String,
    pub max_floating: usize,
    /// seconds a krathong keeps floating before fading out
    pub float_seconds: f32,
    /// gift -> action for this mode ("pray" = ลอยกระทง+ขอเลข, "siamsi" = อธิษฐาน+คำอวยพร). Empty = use [gifts]
    pub gifts: BTreeMap<String, String>,
    pub blessings: Vec<String>,
}
impl Default for KrathongCfg {
    fn default() -> Self {
        Self { hint_text: String::new(), wish_title: "คำอวยพรสำหรับ".into(), max_floating: 14, float_seconds: 120.0, gifts: BTreeMap::new(), blessings: vec![] }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CatCfg {
    pub hint_text: String,
    pub thanks_title: String,
    /// gift -> action for this mode (both "pray" and "siamsi" feed the cat then show a thank-you card; no lucky numbers). Empty = use [gifts]
    pub gifts: BTreeMap<String, String>,
    pub thanks: Vec<String>,
    /// how many cats may be on screen at once (the one eating + the line waiting behind)
    pub max_visible: usize,
}
impl Default for CatCfg {
    fn default() -> Self {
        Self { hint_text: String::new(), thanks_title: "คำขอบคุณจากแมวถึง".into(), gifts: BTreeMap::new(), thanks: vec![], max_visible: 5 }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DanceCfg {
    pub hint_text: String,
    pub card_title: String,
    pub guide_label: String,
    pub max_visible: usize,
    /// "auto" = สุ่มจากเวทีที่ปลดล็อกตามขั้น (เวทีใหญ่มีน้ำหนักมากกว่า)
    /// "random" = สุ่มเท่า ๆ กันจากเวทีที่ปลดล็อก
    /// หรือระบุคีย์เวที เช่น "floor", "stage", "cloud"
    pub venue: String,
    /// คีย์เวทีที่เปิดใช้ ว่าง = ใช้ทั้งหมด
    pub venues: Vec<String>,
    pub challenges: Vec<String>,
    pub combo_joiner: String,
    /// วินาทีของนับถอยหลังต่อขั้น (1..=5)
    pub countdown: Vec<f32>,
    pub gifts: BTreeMap<String, String>,
}
impl Default for DanceCfg {
    fn default() -> Self {
        Self {
            hint_text: String::new(),
            card_title: "ภารกิจเต้น!".into(),
            guide_label: "ตัวชี้ท่า".into(),
            max_visible: 6,
            venue: "auto".into(),
            venues: vec![],
            challenges: vec![],
            combo_joiner: " แล้วต่อด้วย ".into(),
            countdown: vec![8.0, 10.0, 14.0, 18.0, 24.0],
            gifts: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DunkCfg {
    pub hint_text: String,
    pub card_title: String,
    pub dunk_banner: String,
    /// ใช้ `{name}` แทนชื่อผู้ส่ง
    pub splash_text: String,
    pub meter_target: u32,
    pub max_visible: usize,
    /// ขั้น (1-based) ที่ดังก์ทันที ค่าเริ่มต้น 5
    pub instant_dunk_tier: usize,
    pub meter_add: Vec<u32>,
    pub balls: Vec<u32>,
    pub gifts: BTreeMap<String, String>,
}
impl Default for DunkCfg {
    fn default() -> Self {
        Self {
            hint_text: String::new(),
            card_title: "ตกน้ำ!".into(),
            dunk_banner: "ดังก์! DUNK!".into(),
            splash_text: "SPLASH! ตกน้ำเพราะ {name}".into(),
            meter_target: 10,
            max_visible: 5,
            instant_dunk_tier: 5,
            meter_add: vec![1, 2, 4, 7, 10],
            balls: vec![1, 2, 4, 7, 3],
            gifts: BTreeMap::new(),
        }
    }
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
    /// "scene" = ฉากเต็ม (full backdrop), "transparent" = โปร่งใส (only characters, tags, cards, fireworks
    /// over the camera), "green" = chroma key
    pub background: String,
    /// transparent mode: keep small ground props (shrine / river line / bowl) while a gift is playing
    pub props: bool,
    pub always_on_top: bool,
    pub click_through: bool,
    pub show_hint: bool,
    pub hint_text: String,
    pub show_status: bool,
    /// when no gift is playing: "still" = still picture (default), "ambient" = subtle motion, "hidden" = fully transparent
    pub idle: String,
    /// how many characters may be on screen at once (the one at the shrine + the line waiting behind)
    pub max_visible: usize,
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
            node_command: String::new(),
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
            props: true,
            always_on_top: true,
            click_through: false,
            show_hint: true,
            hint_text: String::new(),
            show_status: true,
            idle: "still".into(),
            max_visible: 5,
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
            gifts: GiftsCfg::default(),
            http: HttpCfg::default(),
            window: WindowCfg::default(),
            numbers: NumbersCfg::default(),
            timing: TimingCfg::default(),
            tiers: vec![],
            siamsi: SiamsiCfg::default(),
            theme: ThemeCfg::default(),
            krathong: KrathongCfg::default(),
            cat: CatCfg::default(),
            dance: DanceCfg::default(),
            dunk: DunkCfg::default(),
            update: UpdateCfg::default(),
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
        if cfg.krathong.blessings.is_empty() {
            cfg.krathong.blessings = def.krathong.blessings.clone();
        }
        if cfg.krathong.hint_text.is_empty() {
            cfg.krathong.hint_text = def.krathong.hint_text.clone();
        }
        cfg.krathong.max_floating = cfg.krathong.max_floating.clamp(1, 60);
        if cfg.cat.thanks.is_empty() {
            cfg.cat.thanks = def.cat.thanks.clone();
        }
        if cfg.cat.hint_text.is_empty() {
            cfg.cat.hint_text = def.cat.hint_text.clone();
        }
        if cfg.cat.thanks_title.is_empty() {
            cfg.cat.thanks_title = def.cat.thanks_title.clone();
        }
        cfg.cat.max_visible = cfg.cat.max_visible.clamp(1, 12);
        if cfg.dance.challenges.is_empty() {
            cfg.dance.challenges = def.dance.challenges.clone();
        }
        if cfg.dance.hint_text.is_empty() {
            cfg.dance.hint_text = def.dance.hint_text.clone();
        }
        if cfg.dance.card_title.is_empty() {
            cfg.dance.card_title = def.dance.card_title.clone();
        }
        if cfg.dance.guide_label.is_empty() {
            cfg.dance.guide_label = def.dance.guide_label.clone();
        }
        if cfg.dance.combo_joiner.is_empty() {
            cfg.dance.combo_joiner = def.dance.combo_joiner.clone();
        }
        if cfg.dance.countdown.len() < 5 {
            cfg.dance.countdown = def.dance.countdown.clone();
        }
        cfg.dance.max_visible = cfg.dance.max_visible.clamp(1, 12);
        if cfg.dunk.hint_text.is_empty() {
            cfg.dunk.hint_text = def.dunk.hint_text.clone();
        }
        if cfg.dunk.card_title.is_empty() {
            cfg.dunk.card_title = def.dunk.card_title.clone();
        }
        if cfg.dunk.dunk_banner.is_empty() {
            cfg.dunk.dunk_banner = def.dunk.dunk_banner.clone();
        }
        if cfg.dunk.splash_text.is_empty() {
            cfg.dunk.splash_text = def.dunk.splash_text.clone();
        }
        if cfg.dunk.meter_add.len() < 5 {
            cfg.dunk.meter_add = def.dunk.meter_add.clone();
        }
        if cfg.dunk.balls.len() < 5 {
            cfg.dunk.balls = def.dunk.balls.clone();
        }
        if cfg.dunk.instant_dunk_tier == 0 || cfg.dunk.instant_dunk_tier > 5 {
            cfg.dunk.instant_dunk_tier = 5;
        }
        cfg.dunk.meter_target = cfg.dunk.meter_target.max(1);
        cfg.dunk.max_visible = cfg.dunk.max_visible.clamp(1, 12);
        let mode = cfg.theme.mode.trim().to_lowercase();
        cfg.theme.mode = if mode == "krathong" || mode.contains("กระทง") {
            "krathong".into()
        } else if mode == "cat" || mode.contains("แมว") || mode.contains("อาหาร") {
            "cat".into()
        } else if mode == "dance" || mode.contains("เต้น") {
            "dance".into()
        } else if mode == "dunk" || mode.contains("ตกน้ำ") || mode.contains("ปาตกน้ำ") || mode.contains("ดังก์") {
            "dunk".into()
        } else {
            "shrine".into()
        };
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

    /// Lookup gift -> action string ("pray"/"siamsi"/"none") for the active theme.
    pub fn action_for_gift(&self, name: &str, id: u64) -> String {
        let lname = name.trim().to_lowercase();
        let map = if self.theme.mode == "krathong" && !self.krathong.gifts.is_empty() {
            &self.krathong.gifts
        } else if self.theme.mode == "cat" && !self.cat.gifts.is_empty() {
            &self.cat.gifts
        } else if self.theme.mode == "dance" && !self.dance.gifts.is_empty() {
            &self.dance.gifts
        } else if self.theme.mode == "dunk" && !self.dunk.gifts.is_empty() {
            &self.dunk.gifts
        } else {
            &self.gifts.map
        };
        for (k, v) in map {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_parses_and_is_thai_only() {
        let c = Config::builtin();
        assert_eq!(c.siamsi.fortunes.len(), 28);
        assert_eq!(c.krathong.blessings.len(), 28);
        assert!(c.cat.thanks.len() >= 14, "cat thanks {}", c.cat.thanks.len());
        assert!(c.dance.challenges.len() >= 8, "dance challenges {}", c.dance.challenges.len());
        assert_eq!(c.dunk.meter_target, 10);
        assert_eq!(c.dance.venue, "auto");
        assert_eq!(c.tiers.len(), 5);
        assert_eq!(c.cat.max_visible, 5);
        // no CJK ideographs anywhere in the shipped texts
        for t in c.siamsi.fortunes.iter().chain(c.krathong.blessings.iter()).chain(c.cat.thanks.iter()) {
            assert!(!t.chars().any(|ch| ('\u{4E00}'..='\u{9FFF}').contains(&ch)), "CJK in {t}");
        }
    }
    #[test]
    fn tiers_thresholds() {
        let c = Config::builtin();
        assert_eq!(c.tier_for(0), 0);
        assert_eq!(c.tier_for(9), 0);
        assert_eq!(c.tier_for(10), 1);
        assert_eq!(c.tier_for(99), 1);
        assert_eq!(c.tier_for(100), 2);
        assert_eq!(c.tier_for(999), 2);
        assert_eq!(c.tier_for(1000), 3);
        assert_eq!(c.tier_for(9999), 3);
        assert_eq!(c.tier_for(10000), 4);
        assert_eq!(c.tier_for(5_000_000), 4);
        // diamonds x repeat: Rose(1) x 150 = 150 -> tier 3 (index 2)
        assert_eq!(c.tier_for(1 * 150), 2);
    }
    #[test]
    fn gift_mapping() {
        let mut c = Config::builtin();
        assert_eq!(c.action_for_gift("Rose", 0), "pray");
        assert_eq!(c.action_for_gift("rose", 0), "pray");
        assert_eq!(c.action_for_gift("", 5655), "pray");
        assert_eq!(c.action_for_gift("TikTok", 0), "siamsi");
        assert_eq!(c.action_for_gift("Unknown Gift", 0), "pray"); // default_action
        c.tiktok.default_action = "none".into();
        assert_eq!(c.action_for_gift("Unknown Gift", 0), "none");
        // krathong mode with its own table
        c.theme.mode = "krathong".into();
        c.krathong.gifts.insert("Rose".into(), "siamsi".into());
        assert_eq!(c.action_for_gift("Rose", 0), "siamsi");
        // cat mode with its own table
        c.theme.mode = "cat".into();
        c.cat.gifts.insert("Rose".into(), "siamsi".into());
        assert_eq!(c.action_for_gift("Rose", 0), "siamsi");
    }
    #[test]
    fn save_reload_roundtrip() {
        let mut c = Config::builtin();
        c.tiktok.username = "ทดสอบ_user".into();
        c.window.background = "green".into();
        c.window.layout = "full".into();
        c.theme.mode = "cat".into();
        c.tiers[2].min_diamonds = 222;
        c.gifts.map.insert("Galaxy".into(), "siamsi".into());
        c.krathong.blessings = vec!["ขอให้มีความสุข".into()];
        c.cat.thanks = vec!["เหมียว ๆ ขอบคุณนะ".into()];
        let dir = std::env::temp_dir().join(format!("tsl_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("config.toml");
        c.save(&p).unwrap();
        let r = Config::load(&p);
        assert_eq!(r.tiktok.username, "ทดสอบ_user");
        assert_eq!(r.window.background, "green");
        assert_eq!(r.window.layout, "full");
        assert_eq!(r.theme.mode, "cat");
        assert_eq!(r.tiers[2].min_diamonds, 222);
        assert_eq!(r.gifts.map.get("Galaxy").map(|s| s.as_str()), Some("siamsi"));
        assert_eq!(r.krathong.blessings, vec!["ขอให้มีความสุข".to_string()]);
        assert_eq!(r.cat.thanks, vec!["เหมียว ๆ ขอบคุณนะ".to_string()]);
        assert_eq!(r.window_size(), (540.0, 960.0));
        let _ = std::fs::remove_dir_all(dir);
    }
}
