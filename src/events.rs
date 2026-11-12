//! Event sources: TikTok bridge (Node tiktok-live-connector), local HTTP (TikFinity / Streamer.bot / curl)
//! and keyboard (handled inside the app). All sources implement [`EventSource`] and push [`Incoming`].
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Pray,
    Siamsi,
}

impl Action {
    pub fn parse(s: &str) -> Option<Action> {
        match s.trim().to_lowercase().as_str() {
            "pray" | "wai" | "number" | "lucky" | "ไหว้พระ" | "ไหว้" | "1" => Some(Action::Pray),
            "siamsi" | "fortune" | "seamsi" | "เซียมซี" | "2" => Some(Action::Siamsi),
            _ => None,
        }
    }
}

/// A gift as reported by any connector.
#[derive(Debug, Clone, Default)]
pub struct GiftIn {
    pub user: String,
    pub gift_id: u64,
    pub gift_name: String,
    pub diamonds: u64,
    pub repeat: u32,
}

/// A request that bypasses gift mapping (test keys / ?action=...).
#[derive(Debug, Clone)]
pub struct DirectIn {
    pub action: Action,
    pub user: String,
    pub diamonds: u64,
    #[allow(dead_code)]
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum ControlCmd {
    ClickThrough(Option<bool>), // None = toggle
    Background(String),
    ClearQueue,
}

#[derive(Debug, Clone)]
pub enum Incoming {
    Gift(GiftIn),
    Direct(DirectIn),
    Control(ControlCmd),
    Status(String),
}

/// Sender wrapper that also wakes the UI.
#[derive(Clone)]
pub struct EventSink {
    tx: Sender<Incoming>,
    ctx: egui::Context,
}
impl EventSink {
    pub fn new(tx: Sender<Incoming>, ctx: egui::Context) -> Self {
        Self { tx, ctx }
    }
    pub fn send(&self, ev: Incoming) {
        let _ = self.tx.send(ev);
        self.ctx.request_repaint();
    }
}

/// Anything that can feed live events into the overlay.
pub trait EventSource: Send {
    fn name(&self) -> String;
    /// Start in a background thread; must not block.
    fn start(self: Box<Self>, sink: EventSink);
}

// ---------------------------------------------------------------------------
// JSON helpers (accept many field-name styles: ours, tiktok-live-connector, TikFinity, Streamer.bot)

fn get_str(v: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(x) = v.get(*k) {
            if let Some(s) = x.as_str() {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            } else if x.is_number() {
                return Some(x.to_string());
            }
        }
    }
    None
}
fn get_u64(v: &Value, keys: &[&str]) -> Option<u64> {
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

const USER_KEYS: &[&str] = &["user", "nickname", "nickName", "username", "uniqueId", "unique_id", "name"];
const GIFT_NAME_KEYS: &[&str] = &["gift_name", "giftName", "gift", "giftname"];
const GIFT_ID_KEYS: &[&str] = &["gift_id", "giftId", "giftid"];
const DIAMOND_KEYS: &[&str] = &["diamonds", "diamond_count", "diamondCount", "coins", "value"];
const REPEAT_KEYS: &[&str] = &["repeat", "repeatCount", "repeat_count", "count", "amount"];

/// Parse a JSON object (from HTTP body or bridge line) into an Incoming event.
pub fn parse_json_event(v: &Value) -> Option<Incoming> {
    // nested user object (tiktok-live-connector raw style)
    let user = v
        .get("user")
        .and_then(|u| if u.is_object() { get_str(u, &["nickname", "uniqueId"]) } else { None })
        .or_else(|| get_str(v, USER_KEYS))
        .unwrap_or_else(|| "ผู้ชม".to_string());
    let typ = get_str(v, &["type", "event"]).unwrap_or_default().to_lowercase();
    if typ == "status" {
        return Some(Incoming::Status(get_str(v, &["status"]).unwrap_or_default()));
    }
    if let Some(a) = get_str(v, &["action"]).and_then(|a| Action::parse(&a)) {
        let diamonds = get_u64(v, DIAMOND_KEYS).unwrap_or(1) * get_u64(v, REPEAT_KEYS).unwrap_or(1).max(1);
        let label = get_str(v, GIFT_NAME_KEYS).unwrap_or_default();
        return Some(Incoming::Direct(DirectIn { action: a, user, diamonds, label }));
    }
    let gift_name = get_str(v, GIFT_NAME_KEYS).unwrap_or_default();
    let gift_id = get_u64(v, GIFT_ID_KEYS).unwrap_or(0);
    if gift_name.is_empty() && gift_id == 0 {
        return None;
    }
    Some(Incoming::Gift(GiftIn {
        user,
        gift_id,
        gift_name,
        diamonds: get_u64(v, DIAMOND_KEYS).unwrap_or(1),
        repeat: get_u64(v, REPEAT_KEYS).unwrap_or(1).clamp(1, 100_000) as u32,
    }))
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                let h = std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok());
                match h {
                    Some(v) => {
                        out.push(v);
                        i += 2;
                    }
                    None => out.push(b'%'),
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn query_to_json(q: &str) -> Value {
    let mut m = serde_json::Map::new();
    for pair in q.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        m.insert(percent_decode(k), Value::String(percent_decode(v)));
    }
    Value::Object(m)
}

// ---------------------------------------------------------------------------
/// Local HTTP endpoint.
///  POST /event  (JSON)   {"action":"pray","user":"Somchai","diamonds":100}
///  POST /event  (JSON)   {"gift_name":"Rose","gift_id":5655,"user":"Somchai","diamonds":1,"repeat":5}
///  GET  /event?action=siamsi&user=Somchai&diamonds=1000
///  GET  /gift?gift_name=Rose&user=Somchai&repeat=3
///  GET  /control?click_through=toggle | background=green | clear=1
///  GET  /health
pub struct HttpSource {
    pub bind: String,
}

impl EventSource for HttpSource {
    fn name(&self) -> String {
        format!("HTTP {}", self.bind)
    }
    fn start(self: Box<Self>, sink: EventSink) {
        std::thread::spawn(move || {
            let server = match tiny_http::Server::http(&self.bind) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[http] เปิดพอร์ต {} ไม่ได้: {e}", self.bind);
                    sink.send(Incoming::Status(format!("HTTP error: {e}")));
                    return;
                }
            };
            eprintln!("[http] ฟังที่ http://{}/event", self.bind);
            for mut req in server.incoming_requests() {
                let url = req.url().to_string();
                let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
                let mut body = String::new();
                let _ = req.as_reader().take(64 * 1024).read_to_string(&mut body);
                let mut v = query_to_json(query);
                if !body.trim().is_empty() {
                    if let Ok(Value::Object(b)) = serde_json::from_str::<Value>(&body) {
                        if let Value::Object(m) = &mut v {
                            m.extend(b);
                        }
                    } else if body.contains('=') {
                        if let (Value::Object(m), Value::Object(b)) = (&mut v, query_to_json(body.trim())) {
                            m.extend(b);
                        }
                    }
                }
                let (code, msg) = match path.trim_end_matches('/') {
                    "/health" | "" => (200, "ok".to_string()),
                    "/control" => {
                        let mut done = vec![];
                        if let Some(c) = get_str(&v, &["click_through"]) {
                            let val = match c.as_str() {
                                "1" | "true" | "on" => Some(true),
                                "0" | "false" | "off" => Some(false),
                                _ => None,
                            };
                            sink.send(Incoming::Control(ControlCmd::ClickThrough(val)));
                            done.push("click_through");
                        }
                        if let Some(b) = get_str(&v, &["background"]) {
                            sink.send(Incoming::Control(ControlCmd::Background(b)));
                            done.push("background");
                        }
                        if get_str(&v, &["clear"]).is_some() {
                            sink.send(Incoming::Control(ControlCmd::ClearQueue));
                            done.push("clear");
                        }
                        (200, format!("ok {:?}", done))
                    }
                    "/event" | "/gift" | "/webhook" => match parse_json_event(&v) {
                        Some(ev) => {
                            sink.send(ev);
                            (200, "queued".to_string())
                        }
                        None => (400, "need action=pray|siamsi or gift_name/gift_id".to_string()),
                    },
                    _ => (404, "not found".to_string()),
                };
                let resp = tiny_http::Response::from_string(msg)
                    .with_status_code(code)
                    .with_header(tiny_http::Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(resp);
            }
        });
    }
}

/// GUI apps (macOS .app, Windows double-click) often don't get the shell PATH: probe usual Node locations.
pub fn find_node(cmd: &str) -> PathBuf {
    if cmd != "node" {
        return PathBuf::from(cmd);
    }
    let mut cands: Vec<PathBuf> = vec![];
    #[cfg(windows)]
    {
        for v in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Some(p) = std::env::var_os(v) {
                cands.push(PathBuf::from(&p).join("nodejs").join("node.exe"));
                cands.push(PathBuf::from(&p).join("Programs").join("nodejs").join("node.exe"));
            }
        }
    }
    #[cfg(not(windows))]
    {
        for p in ["/opt/homebrew/bin/node", "/usr/local/bin/node", "/usr/bin/node"] {
            cands.push(PathBuf::from(p));
        }
        if let Some(h) = std::env::var_os("HOME") {
            let nvm = PathBuf::from(h).join(".nvm/versions/node");
            if let Ok(rd) = std::fs::read_dir(&nvm) {
                let mut v: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path().join("bin/node")).collect();
                v.sort();
                cands.extend(v.into_iter().rev());
            }
        }
    }
    // PATH first, then well-known places
    if let Some(paths) = std::env::var_os("PATH") {
        for d in std::env::split_paths(&paths) {
            let p = d.join(if cfg!(windows) { "node.exe" } else { "node" });
            if p.exists() {
                return p;
            }
        }
    }
    cands.into_iter().find(|p| p.exists()).unwrap_or_else(|| PathBuf::from("node"))
}

// ---------------------------------------------------------------------------
/// Shared, restartable state of the TikTok bridge (the settings window edits it).
pub struct BridgeState {
    pub username: String,
    pub sign_key: String,
    pub node: String,
    pub script: PathBuf,
    pub enabled: bool,
    pub reconnect: Duration,
    pub generation: u64,
    pub child: Option<Child>,
    pub status: String,
}
pub type BridgeCtl = Arc<Mutex<BridgeState>>;

/// Apply new settings and force a reconnect.
pub fn restart_bridge(ctl: &BridgeCtl, username: &str, sign_key: &str, node: &str, enabled: bool) {
    let mut st = ctl.lock().unwrap();
    st.username = username.trim().trim_start_matches('@').to_string();
    st.sign_key = sign_key.trim().to_string();
    st.node = if node.trim().is_empty() { "node".into() } else { node.trim().to_string() };
    st.enabled = enabled;
    st.generation += 1;
    if let Some(c) = st.child.as_mut() {
        let _ = c.kill();
    }
}

/// Runs `node bridge/bridge.js <username>` (tiktok-live-connector) and reads JSON lines from stdout.
/// Restarts automatically when it exits (offline, disconnected, TikTok changes...).
pub struct TikTokBridgeSource {
    pub ctl: BridgeCtl,
}

impl EventSource for TikTokBridgeSource {
    fn name(&self) -> String {
        format!("TikTok @{} (tiktok-live-connector)", self.ctl.lock().unwrap().username)
    }
    fn start(self: Box<Self>, sink: EventSink) {
        std::thread::spawn(move || {
            let ctl = self.ctl;
            let status = |s: String| {
                let mut st = ctl.lock().unwrap();
                if st.status != s {
                    eprintln!("[tiktok] {s}");
                    st.status = s.clone();
                    drop(st);
                    sink.send(Incoming::Status(s));
                }
            };
            loop {
                let (user, key, node, script, enabled, generation, reconnect) = {
                    let st = ctl.lock().unwrap();
                    (st.username.clone(), st.sign_key.clone(), st.node.clone(), st.script.clone(), st.enabled, st.generation, st.reconnect)
                };
                if !enabled || user.is_empty() {
                    status("ยังไม่ได้เชื่อม TikTok (ใส่ชื่อผู้ใช้ในหน้าตั้งค่า)".into());
                    std::thread::sleep(Duration::from_millis(500));
                    continue;
                }
                status(format!("กำลังเชื่อมต่อ @{user} ..."));
                let mut cmd = Command::new(find_node(&node));
                cmd.arg(&script).arg(&user);
                if !key.is_empty() {
                    cmd.arg("--sign-key").arg(&key);
                }
                if let Some(dir) = script.parent() {
                    if !dir.as_os_str().is_empty() {
                        cmd.current_dir(dir);
                    }
                }
                cmd.stdout(Stdio::piped()).stderr(Stdio::inherit()).stdin(Stdio::null());
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
                }
                match cmd.spawn() {
                    Ok(mut child) => {
                        let out = child.stdout.take();
                        ctl.lock().unwrap().child = Some(child);
                        if let Some(out) = out {
                            for line in BufReader::new(out).lines().map_while(Result::ok) {
                                let line = line.trim();
                                if line.is_empty() {
                                    continue;
                                }
                                match serde_json::from_str::<Value>(line) {
                                    Ok(v) => match parse_json_event(&v) {
                                        Some(Incoming::Status(s)) => {
                                            let s = match s.as_str() {
                                                "connected" => format!("เชื่อมต่อแล้ว @{user}"),
                                                "connecting" => format!("กำลังเชื่อมต่อ @{user} ..."),
                                                "connect_failed" => format!("เชื่อมต่อไม่ได้ (@{user} ไม่ได้ไลฟ์อยู่?)"),
                                                "stream_end" => "ไลฟ์จบแล้ว".to_string(),
                                                other => other.to_string(),
                                            };
                                            status(s);
                                        }
                                        Some(ev) => sink.send(ev),
                                        None => {}
                                    },
                                    Err(_) => eprintln!("[bridge] {line}"),
                                }
                            }
                        }
                        let child = ctl.lock().unwrap().child.take();
                        if let Some(mut c) = child {
                            let _ = c.wait();
                        }
                    }
                    Err(e) => {
                        eprintln!("[tiktok] รัน `{node}` ไม่ได้ ({e}) - ติดตั้ง Node.js แล้วรัน `npm install` ในโฟลเดอร์ bridge");
                        status("ไม่พบ Node.js - ดู README".into());
                    }
                }
                // wait before reconnect unless settings changed
                let mut waited = Duration::ZERO;
                let mut announced = false;
                while waited < reconnect {
                    if ctl.lock().unwrap().generation != generation {
                        break;
                    }
                    if !announced {
                        let cur = ctl.lock().unwrap().status.clone();
                        if !cur.starts_with("เชื่อมต่อไม่ได้") && !cur.starts_with("ไม่พบ") {
                            status(format!("หลุด - ต่อใหม่ใน {}s", reconnect.as_secs()));
                        }
                        announced = true;
                    }
                    std::thread::sleep(Duration::from_millis(250));
                    waited += Duration::from_millis(250);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decode() {
        assert_eq!(percent_decode("a%20b+c"), "a b c");
        assert_eq!(percent_decode("%E0%B8%81"), "ก");
        assert_eq!(percent_decode("100%"), "100%");
    }
    #[test]
    fn parse() {
        let v = query_to_json("action=siamsi&user=%E0%B8%81&diamonds=5");
        match parse_json_event(&v) {
            Some(Incoming::Direct(d)) => {
                assert_eq!(d.action, Action::Siamsi);
                assert_eq!(d.user, "ก");
                assert_eq!(d.diamonds, 5);
            }
            other => panic!("{other:?}"),
        }
        let v: Value = serde_json::from_str(r#"{"type":"gift","user":"A","gift_id":5655,"gift_name":"Rose","diamonds":1,"repeat":7}"#).unwrap();
        match parse_json_event(&v) {
            Some(Incoming::Gift(g)) => assert_eq!((g.gift_id, g.repeat), (5655, 7)),
            other => panic!("{other:?}"),
        }
    }
}
