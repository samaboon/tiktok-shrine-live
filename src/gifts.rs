//! TikTok gift catalog: parse real lists, bundle a cited snapshot, cache to disk, fetch icons.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Bundled snapshot shipped with the app (see `assets/gifts/catalog.json` for source + date).
pub const BUNDLED_CATALOG_JSON: &str = include_str!("../assets/gifts/catalog.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GiftInfo {
    pub id: u64,
    pub name: String,
    pub diamond_count: u64,
    /// HTTPS URL of the sticker image (webp/png/jpeg). May be empty.
    #[serde(default)]
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftCatalog {
    pub source: String,
    #[serde(default)]
    pub source_url: String,
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub language: String,
    /// RFC 3339 timestamp of when this list was fetched.
    pub fetched_at: String,
    pub count: usize,
    pub gifts: Vec<GiftInfo>,
    /// True when this list came from a live `fetchAvailableGifts` call (vs the shipped snapshot).
    #[serde(default)]
    pub from_live: bool,
    /// Last fetch error, if any (catalog may still hold a previous/bundled list).
    #[serde(default)]
    pub last_error: String,
}

impl Default for GiftCatalog {
    fn default() -> Self {
        Self {
            source: String::new(),
            source_url: String::new(),
            api_url: String::new(),
            region: String::new(),
            language: String::new(),
            fetched_at: String::new(),
            count: 0,
            gifts: vec![],
            from_live: false,
            last_error: String::new(),
        }
    }
}

impl GiftCatalog {
    pub fn bundled() -> Result<Self, String> {
        parse_gift_list_json(BUNDLED_CATALOG_JSON).map(|mut c| {
            c.from_live = false;
            c
        })
    }

    pub fn load_preferred(cache_dir: &Path) -> Self {
        let bundled = Self::bundled().ok();
        let cached = load_cache(cache_dir).ok();
        match (bundled, cached) {
            (Some(b), Some(c))
                if !c.gifts.is_empty() && catalog_newer(&c.fetched_at, &b.fetched_at) =>
            {
                c
            }
            (Some(b), _) => b,
            (None, Some(c)) => c,
            (None, None) => Self::default(),
        }
    }

    pub fn save_cache(&self, cache_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(cache_dir).map_err(|e| e.to_string())?;
        let p = cache_dir.join("catalog.json");
        let body = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(p, body).map_err(|e| e.to_string())
    }

    pub fn replace_from_msg(&mut self, msg: GiftListMsg, cache_dir: &Path) {
        if let Some(err) = msg.error.filter(|s| !s.is_empty()) {
            self.last_error = err;
            return;
        }
        if msg.gifts.is_empty() {
            self.last_error = "ตัวเชื่อมส่งรายการว่างมา".into();
            return;
        }
        self.gifts = dedupe_gifts(msg.gifts);
        self.count = self.gifts.len();
        self.source = if msg.source.is_empty() {
            "tiktok-live-connector fetchAvailableGifts (runtime)".into()
        } else {
            msg.source
        };
        self.fetched_at = if msg.fetched_at.is_empty() {
            now_rfc3339()
        } else {
            msg.fetched_at
        };
        self.from_live = true;
        self.last_error.clear();
        let _ = self.save_cache(cache_dir);
    }

    pub fn find(&self, id: u64) -> Option<&GiftInfo> {
        self.gifts.iter().find(|g| g.id == id)
    }

    pub fn find_by_key(&self, key: &str) -> Option<&GiftInfo> {
        let k = key.trim();
        if let Ok(id) = k.parse::<u64>() {
            if let Some(g) = self.find(id) {
                return Some(g);
            }
        }
        let l = k.to_lowercase();
        self.gifts.iter().find(|g| g.name.eq_ignore_ascii_case(&l))
    }
}

/// Message the Node bridge emits as `{ "type": "gift_list", ... }`.
#[derive(Debug, Clone, Default)]
pub struct GiftListMsg {
    pub gifts: Vec<GiftInfo>,
    pub source: String,
    pub fetched_at: String,
    pub error: Option<String>,
}

pub fn parse_gift_list_json(s: &str) -> Result<GiftCatalog, String> {
    let v: serde_json::Value =
        serde_json::from_str(s).map_err(|e| format!("gift list JSON: {e}"))?;
    parse_gift_list_value(&v)
}

pub fn parse_gift_list_value(v: &serde_json::Value) -> Result<GiftCatalog, String> {
    if v.get("gifts").is_some() || v.get("data").is_some() || v.get("source").is_some() {
        let (gifts, fetched_at, source, source_url, api_url, region, language) =
            extract_catalog_fields(v)?;
        let gifts = dedupe_gifts(gifts);
        return Ok(GiftCatalog {
            source,
            source_url,
            api_url,
            region,
            language,
            fetched_at,
            count: gifts.len(),
            gifts,
            from_live: false,
            last_error: String::new(),
        });
    }
    if v.is_array() {
        let gifts = dedupe_gifts(parse_gift_array(v)?);
        return Ok(GiftCatalog {
            source: "array".into(),
            fetched_at: String::new(),
            count: gifts.len(),
            gifts,
            ..GiftCatalog::default()
        });
    }
    Err("unrecognized gift list shape".into())
}

pub fn parse_gift_list_msg(v: &serde_json::Value) -> GiftListMsg {
    let error = v
        .get("error")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let gifts = match extract_gifts_array(v) {
        Ok(g) => dedupe_gifts(g),
        Err(_) => vec![],
    };
    GiftListMsg {
        gifts,
        source: v
            .get("source")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        fetched_at: v
            .get("fetched_at")
            .or_else(|| v.get("fetchedAt"))
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        error,
    }
}

fn extract_catalog_fields(
    v: &serde_json::Value,
) -> Result<
    (
        Vec<GiftInfo>,
        String,
        String,
        String,
        String,
        String,
        String,
    ),
    String,
> {
    let gifts = extract_gifts_array(v)?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    Ok((
        gifts,
        s("fetched_at"),
        s("source"),
        s("source_url"),
        s("api_url"),
        s("region"),
        s("language"),
    ))
}

fn extract_gifts_array(v: &serde_json::Value) -> Result<Vec<GiftInfo>, String> {
    if let Some(arr) = v.get("gifts").and_then(|x| x.as_array()) {
        return parse_gift_array(&serde_json::Value::Array(arr.clone()));
    }
    if let Some(arr) = v.pointer("/data/gifts").and_then(|x| x.as_array()) {
        return parse_gift_array(&serde_json::Value::Array(arr.clone()));
    }
    if v.is_array() {
        return parse_gift_array(v);
    }
    Err("no gifts array".into())
}

fn parse_gift_array(v: &serde_json::Value) -> Result<Vec<GiftInfo>, String> {
    let arr = v
        .as_array()
        .ok_or_else(|| "gifts is not an array".to_string())?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        if let Some(g) = parse_one_gift(item) {
            out.push(g);
        }
    }
    Ok(out)
}

fn parse_one_gift(v: &serde_json::Value) -> Option<GiftInfo> {
    let id = json_u64(v, &["id", "gift_id", "giftId"])?;
    if id == 0 {
        return None;
    }
    let name = json_str(v, &["name", "gift_name", "giftName"]).unwrap_or_default();
    let diamond_count =
        json_u64(v, &["diamond_count", "diamondCount", "diamonds", "cost"]).unwrap_or(0);
    let image = first_image_url(v).unwrap_or_default();
    Some(GiftInfo {
        id,
        name,
        diamond_count,
        image,
    })
}

fn first_image_url(v: &serde_json::Value) -> Option<String> {
    if let Some(s) = json_str(
        v,
        &["image", "image_url", "imageUrl", "icon_url", "iconUrl"],
    ) {
        if s.starts_with("https://") || s.starts_with("http://") {
            return Some(s);
        }
    }
    for path in [
        "/image/url_list/0",
        "/image/urlList/0",
        "/icon/url_list/0",
        "/icon/urlList/0",
        "/image/url",
        "/icon/url",
    ] {
        if let Some(s) = v.pointer(path).and_then(|x| x.as_str()) {
            if s.starts_with("http") {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn json_str(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn json_u64(v: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(n) = x.as_u64() {
                return Some(n);
            }
            if let Some(f) = x.as_f64() {
                return Some(f.max(0.0) as u64);
            }
            if let Some(s) = x.as_str() {
                if let Ok(n) = s.trim().parse::<u64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn dedupe_gifts(gifts: Vec<GiftInfo>) -> Vec<GiftInfo> {
    let mut seen = HashMap::new();
    let mut out = Vec::with_capacity(gifts.len());
    for g in gifts {
        if g.id == 0 {
            continue;
        }
        if seen.insert(g.id, ()).is_none() {
            out.push(g);
        }
    }
    out
}

fn catalog_newer(a: &str, b: &str) -> bool {
    if a.is_empty() {
        return false;
    }
    if b.is_empty() {
        return true;
    }
    a >= b
}

fn now_rfc3339() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let (y, m, d) = civil_from_days(days);
    let hh = rem / 3600;
    let mm = (rem % 3600) / 60;
    let ss = rem % 60;
    format!("{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    // Howard Hinnant civil_from_days (Unix epoch day 0 = 1970-01-01)
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn load_cache(cache_dir: &Path) -> Result<GiftCatalog, String> {
    let p = cache_dir.join("catalog.json");
    let s = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
    parse_gift_list_json(&s)
}

pub fn cache_dir(base: &Path) -> PathBuf {
    base.join("gifts-cache")
}

/// Upsert a mapping row. Prefers an existing name or id key; otherwise stores the gift id.
pub fn upsert_mapping(rows: &mut Vec<(String, String)>, gift: &GiftInfo, action: &str) {
    let action = action.to_string();
    let id_key = gift.id.to_string();
    if let Some((_, act)) = rows.iter_mut().find(|(k, _)| k.trim() == id_key) {
        *act = action;
        return;
    }
    if !gift.name.is_empty() {
        if let Some((_, act)) = rows
            .iter_mut()
            .find(|(k, _)| k.trim().eq_ignore_ascii_case(&gift.name))
        {
            *act = action;
            return;
        }
    }
    rows.push((id_key, action));
}

// --------------------------------------------------------------------------- images

pub struct IconReady {
    pub id: u64,
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// Background downloader + disk cache for gift sticker images.
pub struct IconFetcher {
    req_tx: Sender<(u64, String)>,
    rx: Receiver<IconReady>,
    pending: Arc<Mutex<HashSet<u64>>>,
}

impl IconFetcher {
    pub fn new(cache_dir: PathBuf) -> Self {
        let (req_tx, req_rx) = mpsc::channel::<(u64, String)>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let pending = Arc::new(Mutex::new(HashSet::new()));
        let pending2 = pending.clone();
        std::thread::Builder::new()
            .name("gift-icons".into())
            .spawn(move || icon_worker(cache_dir, req_rx, ready_tx, pending2))
            .ok();
        Self {
            req_tx,
            rx: ready_rx,
            pending,
        }
    }

    pub fn request(&self, id: u64, url: &str) {
        if url.is_empty() {
            return;
        }
        let mut p = self.pending.lock().unwrap();
        if p.contains(&id) {
            return;
        }
        p.insert(id);
        drop(p);
        let _ = self.req_tx.send((id, url.to_string()));
    }

    pub fn try_recv(&self) -> Vec<IconReady> {
        let mut out = Vec::new();
        while let Ok(icon) = self.rx.try_recv() {
            self.pending.lock().unwrap().remove(&icon.id);
            out.push(icon);
        }
        out
    }
}

fn icon_worker(
    cache_dir: PathBuf,
    rx: Receiver<(u64, String)>,
    tx: Sender<IconReady>,
    pending: Arc<Mutex<HashSet<u64>>>,
) {
    let img_dir = cache_dir.join("img");
    let _ = std::fs::create_dir_all(&img_dir);
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(12))
        .user_agent("tiktok-shrine-live/0.2 (gift icon cache)")
        .build();
    while let Ok((id, url)) = rx.recv() {
        match load_or_download(&agent, &img_dir, id, &url) {
            Ok(icon) => {
                if tx.send(icon).is_err() {
                    break;
                }
            }
            Err(_) => {
                pending.lock().unwrap().remove(&id);
            }
        }
    }
}

fn load_or_download(
    agent: &ureq::Agent,
    img_dir: &Path,
    id: u64,
    url: &str,
) -> Result<IconReady, String> {
    if let Some(icon) = load_cached_icon(img_dir, id) {
        return Ok(icon);
    }
    let resp = agent.get(url).call().map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    resp.into_reader()
        .take(2_000_000)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() < 16 {
        return Err("empty image".into());
    }
    // cache raw bytes (webp/png/jpeg) under a stable name
    let ext = sniff_ext(&bytes);
    let path = img_dir.join(format!("{id}.{ext}"));
    let _ = std::fs::write(&path, &bytes);
    decode_icon(id, &bytes)
}

fn load_cached_icon(img_dir: &Path, id: u64) -> Option<IconReady> {
    for ext in ["webp", "png", "jpg", "jpeg"] {
        let p = img_dir.join(format!("{id}.{ext}"));
        if let Ok(bytes) = std::fs::read(&p) {
            if let Ok(icon) = decode_icon(id, &bytes) {
                return Some(icon);
            }
        }
    }
    None
}

fn sniff_ext(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "webp"
    } else if bytes.len() >= 8 && bytes[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        "png"
    } else if bytes.len() >= 3 && bytes[0..3] == [0xFF, 0xD8, 0xFF] {
        "jpg"
    } else {
        "webp"
    }
}

fn decode_icon(id: u64, bytes: &[u8]) -> Result<IconReady, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    // shrink very large stickers so the texture stays light
    let img = if img.width() > 128 || img.height() > 128 {
        image::imageops::thumbnail(&img, 96, 96)
    } else {
        img
    };
    let (w, h) = img.dimensions();
    Ok(IconReady {
        id,
        rgba: img.into_raw(),
        w,
        h,
    })
}

use std::io::Read;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_snapshot_is_real_and_complete() {
        let c = GiftCatalog::bundled().expect("bundled catalog must parse");
        assert!(c.count >= 700, "expected a full catalog, got {}", c.count);
        assert_eq!(c.gifts.len(), c.count);
        assert!(c.source.contains("Euler Stream"), "source={}", c.source);
        assert!(
            c.fetched_at.starts_with("2026-11-12"),
            "fetched_at={}",
            c.fetched_at
        );
        let rose = c.find(5655).expect("Rose 5655");
        assert_eq!(rose.name, "Rose");
        assert_eq!(rose.diamond_count, 1);
        assert!(rose.image.starts_with("https://"));
        let tiktok = c.find_by_key("TikTok").expect("TikTok by name");
        assert_eq!(tiktok.id, 5269);
        assert_eq!(tiktok.diamond_count, 1);
        // unique ids
        let mut ids: Vec<u64> = c.gifts.iter().map(|g| g.id).collect();
        let n = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), n);
    }

    #[test]
    fn parses_bridge_message_and_raw_webcast() {
        let bridge = r#"{"type":"gift_list","source":"tiktok-live-connector fetchAvailableGifts","fetched_at":"2026-11-12T00:00:00Z","gifts":[{"id":5655,"name":"Rose","diamond_count":1,"image":"https://example.com/rose.webp"}]}"#;
        let v: serde_json::Value = serde_json::from_str(bridge).unwrap();
        let msg = parse_gift_list_msg(&v);
        assert_eq!(msg.gifts.len(), 1);
        assert_eq!(msg.gifts[0].id, 5655);
        assert_eq!(msg.gifts[0].name, "Rose");
        assert!(msg.error.is_none());

        let raw = r#"{"data":{"gifts":[{"id":"5269","name":"TikTok","diamond_count":1,"image":{"url_list":["https://cdn.example/tiktok.webp"]}},{"id":5269,"name":"TikTok","diamond_count":1,"image":{"url_list":["https://cdn.example/tiktok.webp"]}}]}}"#;
        let c = parse_gift_list_json(raw).unwrap();
        assert_eq!(c.gifts.len(), 1, "duplicate ids must be dropped");
        assert_eq!(c.gifts[0].id, 5269);
        assert_eq!(c.gifts[0].image, "https://cdn.example/tiktok.webp");
    }

    #[test]
    fn upsert_mapping_prefers_existing_name_key() {
        let g = GiftInfo {
            id: 5655,
            name: "Rose".into(),
            diamond_count: 1,
            image: String::new(),
        };
        let mut rows = vec![("Rose".into(), "pray".into())];
        upsert_mapping(&mut rows, &g, "siamsi");
        assert_eq!(rows, vec![("Rose".into(), "siamsi".into())]);
        let g2 = GiftInfo {
            id: 5269,
            name: "TikTok".into(),
            diamond_count: 1,
            image: String::new(),
        };
        upsert_mapping(&mut rows, &g2, "pray");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], ("5269".into(), "pray".into()));
    }

    #[test]
    fn mapping_roundtrip_saved_to_config() {
        let mut c = crate::config::Config::builtin();
        let g = GiftInfo {
            id: 6369,
            name: "Lion".into(),
            diamond_count: 29999,
            image: String::new(),
        };
        let mut rows: Vec<(String, String)> = c
            .gifts
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        upsert_mapping(&mut rows, &g, "siamsi");
        c.gifts = rows.into_iter().collect();
        let dir = std::env::temp_dir().join(format!("tsl_giftmap_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("config.toml");
        c.save(&p).unwrap();
        let r = crate::config::Config::load(&p);
        assert_eq!(r.action_for_gift("Lion", 6369), "siamsi");
        // id-only row also works
        assert_eq!(
            r.gifts
                .get("6369")
                .or_else(|| r.gifts.get("Lion"))
                .map(|s| s.as_str()),
            Some("siamsi")
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn default_mappings_exist_in_bundled_catalog() {
        let cat = GiftCatalog::bundled().expect("bundled snapshot");
        let builtin = crate::config::Config::builtin();
        let example: crate::config::Config = toml::from_str(include_str!("../config.example.toml"))
            .expect("config.example.toml must parse");
        for (label, cfg) in [("config.toml", &builtin), ("config.example.toml", &example)] {
            let sections = [
                ("[gifts]", &cfg.gifts),
                ("[krathong.gifts]", &cfg.krathong.gifts),
                ("[cat.gifts]", &cfg.cat.gifts),
            ];
            for (section, map) in sections {
                for key in map.keys() {
                    assert!(
                        cat.find_by_key(key).is_some(),
                        "{label} {section} key {key:?} is not in the bundled catalog"
                    );
                }
            }
        }
        assert!(cat.find_by_key("Hand Heart").is_some());
        assert!(cat.find_by_key("5660").is_some());
        assert!(cat.find_by_key("TikTok Universe").is_some());
        assert_eq!(
            cat.find_by_key("9101").map(|g| g.name.as_str()),
            Some("TikTok Universe")
        );
        assert!(cat.find_by_key("Hand Hearts").is_none());
        assert!(cat.find_by_key("Universe").is_none());
    }
}
