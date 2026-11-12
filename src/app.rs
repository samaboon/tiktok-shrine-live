//! Overlay app: themes, event queue, animation timeline, camera, fireworks, settings window.
use crate::cat::{self, CatIn, Stage as CatStage};
use crate::config::{digit_label, Config};
use crate::events::{restart_bridge, Action, BridgeCtl, ControlCmd, DirectIn, GiftIn, Incoming};
use crate::krathong as kr;
use crate::scene::*;
use egui::{Align2, Color32, Key, Pos2, Vec2, ViewportBuilder, ViewportCommand, ViewportId, WindowLevel};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

/// Scene themes. Add a variant + its timeline/drawing to support more modes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    /// ไหว้พระขอเลข / เสี่ยงเซียมซี
    Shrine,
    /// ลอยกระทง
    Krathong,
    /// ให้อาหารแมว
    Cat,
}
impl Theme {
    pub fn from_key(s: &str) -> Theme {
        let s = s.trim();
        if s.eq_ignore_ascii_case("krathong") || s.contains("กระทง") {
            Theme::Krathong
        } else if s.eq_ignore_ascii_case("cat") || s.contains("แมว") || s.contains("อาหาร") {
            Theme::Cat
        } else {
            Theme::Shrine
        }
    }
    pub fn key(self) -> &'static str {
        match self {
            Theme::Shrine => "shrine",
            Theme::Krathong => "krathong",
            Theme::Cat => "cat",
        }
    }
    pub fn next(self) -> Theme {
        match self {
            Theme::Shrine => Theme::Krathong,
            Theme::Krathong => Theme::Cat,
            Theme::Cat => Theme::Shrine,
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Theme::Shrine => "ไหว้พระ / เซียมซี",
            Theme::Krathong => "ลอยกระทง",
            Theme::Cat => "ให้อาหารแมว",
        }
    }
    pub fn action_label(self, a: &str) -> &'static str {
        match (self, a) {
            (Theme::Shrine, "pray") => "ไหว้พระขอเลข",
            (Theme::Shrine, "siamsi") => "เสี่ยงเซียมซี",
            (Theme::Krathong, "pray") => "ลอยกระทงขอเลข",
            (Theme::Krathong, "siamsi") => "อธิษฐานรับคำอวยพร",
            (Theme::Cat, "pray") => "ให้อาหารแมว",
            (Theme::Cat, "siamsi") => "แมวขอบคุณ",
            _ => "ไม่ทำอะไร",
        }
    }
}

#[derive(Clone)]
struct StickPhys {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ang: f32,
    va: f32,
    landed: Option<f32>,
}

#[derive(Clone)]
pub struct Appearance {
    pub shirt: Color32,
    pub pants: Color32,
    pub hair: Color32,
    pub style: u8,
    pub seed: f32,
    /// strong colour of the same hue as the shirt: name tag on result cards + marker under the name label
    pub tag: Color32,
}

pub struct Show {
    pub action: Action,
    pub theme: Theme,
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub t: f32,
    pub numbers: Vec<(String, String)>,
    pub stick_no: usize,
    pub text: String,
    pub look: Appearance,
    lane_y: f32,
    bursts: Vec<(f32, u8)>,
    fired: usize,
    trail_acc: f32,
    wish_acc: f32,
    digit_done: Vec<Vec<bool>>,
    stick: Option<StickPhys>,
    released: bool,
    /// waiting-line state: x on screen (None = still off-screen), walk phase, walk amount, time visible
    qx: Option<f32>,
    qphase: f32,
    qmove: f32,
    qage: f32,
    /// where the walk-in to the shrine starts (front of the line, or off-screen)
    start_x: f32,
}

pub struct Floating {
    x: f32,
    y: f32,
    vx: f32,
    age: f32,
    fade: Option<f32>,
    tier: usize,
    name: String,
    seed: f32,
}

/// Screenshot automation (CLI --shot-*)
pub struct ShotPlan {
    pub at: f32,
    pub out: PathBuf,
    pub settings_tab: Option<usize>,
    pub preview_bg: bool,
    pub requested: bool,
    pub done: Arc<AtomicBool>,
    pub frames_after: u32,
}

struct Recorder {
    dir: PathBuf,
    start: f32,
    end: f32,
    requested: usize,
    saved: Arc<AtomicUsize>,
}

#[derive(Clone, Copy, PartialEq)]
struct Applied {
    size: (f32, f32),
    aot: bool,
    click: bool,
}

pub struct ShrineApp {
    pub cfg: Config,
    pub cfg_path: PathBuf,
    rx: Receiver<Incoming>,
    bridge: BridgeCtl,
    queue: VecDeque<Show>,
    current: Option<Show>,
    particles: Vec<Particle>,
    floating: Vec<Floating>,
    rng: fastrand::Rng,
    time: f32,
    scene_time: f32,
    idle_for: f32,
    scene_alpha: f32,
    gap: f32,
    shake: f32,
    /// where the active character's name label was drawn (for the result card pointer)
    banner_at: std::cell::Cell<(f32, f32)>,
    status: String,
    pub running: bool,
    settings_open: bool,
    settings_tab: usize,
    test_tier: usize,
    applied: Option<Applied>,
    sprites: Option<Sprites>,
    assets_dir: PathBuf,
    shot: Option<ShotPlan>,
    rec: Option<Recorder>,
    sim_dt: Option<f32>,
    state: Arc<Mutex<String>>,
    frame: u64,
    shows_started: u64,
    shows_finished: u64,
    gifts_received: u64,
    // settings edit buffers
    cats: CatStage,
    gift_rows: Vec<(String, String)>,
    gift_rows_theme: Theme,
    fortunes_text: String,
    blessings_text: String,
    thanks_text: String,
    flash_msg: (String, f32),
    pos_edit: Option<(f32, f32)>,
    pub updater: crate::updater::Updater,
}

pub struct AppInit {
    pub cfg: Config,
    pub cfg_path: PathBuf,
    pub rx: Receiver<Incoming>,
    pub bridge: BridgeCtl,
    pub assets_dir: PathBuf,
    pub shot: Option<ShotPlan>,
    pub demo: Vec<(Action, usize, String)>,
    pub seed: Option<u64>,
    pub open_settings: bool,
    pub start_stopped: bool,
    pub state: Arc<Mutex<String>>,
    pub theme: Option<String>,
    pub record: Option<(PathBuf, f32, f32, u32)>,
    pub updater: crate::updater::Updater,
}

const WALK: f32 = 3.2;
const OUT: f32 = 2.2;

impl ShrineApp {
    pub fn new(init: AppInit) -> Self {
        let rng = match init.seed {
            Some(s) => fastrand::Rng::with_seed(s),
            None => fastrand::Rng::new(),
        };
        let mut cfg = init.cfg;
        if let Some(t) = &init.theme {
            cfg.theme.mode = Theme::from_key(t).key().into();
        }
        let sim_dt = if let Some((_, _, _, fps)) = &init.record { Some(1.0 / *fps as f32) } else { init.shot.as_ref().map(|_| 1.0 / 60.0) };
        let settings_tab = init.shot.as_ref().and_then(|s| s.settings_tab).unwrap_or(0);
        let theme = Theme::from_key(&cfg.theme.mode);
        let mut app = Self {
            cats: match init.seed {
                Some(s) => CatStage::with_seed(s.wrapping_add(0xCA7)),
                None => CatStage::new(),
            },
            gift_rows: vec![],
            gift_rows_theme: theme,
            fortunes_text: cfg.siamsi.fortunes.join("\n"),
            blessings_text: cfg.krathong.blessings.join("\n"),
            thanks_text: cfg.cat.thanks.join("\n"),
            cfg,
            cfg_path: init.cfg_path,
            rx: init.rx,
            bridge: init.bridge,
            queue: VecDeque::new(),
            current: None,
            particles: vec![],
            floating: vec![],
            rng,
            time: 0.0,
            scene_time: 0.0,
            idle_for: 10.0,
            scene_alpha: 0.0,
            gap: 0.0,
            banner_at: std::cell::Cell::new((PRAY_X, 300.0)),
            shake: 0.0,
            status: String::new(),
            running: !init.start_stopped,
            settings_open: init.open_settings,
            settings_tab,
            test_tier: 0,
            applied: None,
            sprites: None,
            assets_dir: init.assets_dir,
            shot: init.shot,
            rec: init.record.map(|(dir, start, end, _)| Recorder { dir, start, end, requested: 0, saved: Arc::new(AtomicUsize::new(0)) }),
            sim_dt,
            state: init.state,
            frame: 0,
            shows_started: 0,
            shows_finished: 0,
            gifts_received: 0,
            flash_msg: (String::new(), 0.0),
            pos_edit: None,
            updater: init.updater,
        };
        app.load_gift_rows();
        for (a, tier, user) in init.demo {
            let d = app.tier_min(tier).max(1);
            app.enqueue(a, &user, d, tier);
        }
        app
    }

    pub fn theme(&self) -> Theme {
        Theme::from_key(&self.cfg.theme.mode)
    }

    fn load_gift_rows(&mut self) {
        let th = self.theme();
        let map = if th == Theme::Krathong && !self.cfg.krathong.gifts.is_empty() {
            &self.cfg.krathong.gifts
        } else if th == Theme::Cat && !self.cfg.cat.gifts.is_empty() {
            &self.cfg.cat.gifts
        } else {
            &self.cfg.gifts
        };
        self.gift_rows = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        self.gift_rows_theme = th;
    }

    fn tier_min(&self, tier: usize) -> u64 {
        self.cfg.tiers.get(tier).map(|t| t.min_diamonds).unwrap_or(1)
    }

    fn vh(&self) -> f32 {
        let (w, h) = self.cfg.window_size();
        h / w * VW
    }
    fn ground_of(&self, th: Theme) -> f32 {
        match th {
            Theme::Shrine => ground_for(self.vh()),
            Theme::Krathong => kr::bank_y(self.vh()),
            Theme::Cat => cat::floor_y(self.vh()),
        }
    }

    fn enqueue(&mut self, action: Action, user: &str, diamonds: u64, tier_override: usize) {
        let tier = if tier_override < 99 { tier_override.min(self.cfg.tiers.len().saturating_sub(1)) } else { self.cfg.tier_for(diamonds) };
        let tier_name = self.cfg.tiers.get(tier).map(|t| t.name.clone()).unwrap_or_default();
        let digits = self.cfg.numbers.digits.clone();
        let numbers: Vec<(String, String)> = digits
            .iter()
            .map(|&d| {
                let d = d.clamp(1, 6);
                let max = 10u32.pow(d);
                (digit_label(d), format!("{:0width$}", self.rng.u32(0..max), width = d as usize))
            })
            .collect();
        let theme = self.theme();
        let list = match theme {
            Theme::Krathong => self.cfg.krathong.blessings.clone(),
            Theme::Cat => self.cfg.cat.thanks.clone(),
            Theme::Shrine => self.cfg.siamsi.fortunes.clone(),
        };
        let pick = self.rng.usize(0..list.len().max(1));
        let text = list.get(pick).cloned().unwrap_or_default();
        let clean = display_name(user);
        let user = if clean.is_empty() { "ผู้ชม".to_string() } else { truncate(&clean, 18) };
        if theme == Theme::Cat {
            self.cats.enqueue(CatIn {
                action,
                user,
                tier,
                tier_name,
                text,
            });
            return;
        }
        let look = appearance_for(&user);
        let bursts = burst_plan(tier, &mut self.rng);
        let vh = self.vh();
        let wt = kr::water_top(vh);
        let lane_y = lerp(wt + 50.0, vh - 70.0, self.rng.f32());
        let digit_done = numbers.iter().map(|(_, s)| vec![false; s.chars().count()]).collect();
        self.queue.push_back(Show {
            action,
            theme,
            user,
            tier,
            tier_name,
            t: 0.0,
            numbers,
            stick_no: pick + 1,
            text,
            look,
            lane_y,
            bursts,
            fired: 0,
            trail_acc: 0.0,
            wish_acc: 0.0,
            digit_done,
            stick: None,
            released: false,
            qx: None,
            qphase: 0.0,
            qmove: 0.0,
            qage: 0.0,
            start_x: -150.0,
        });
    }

    fn handle_incoming(&mut self, ev: Incoming, ctx: &egui::Context) {
        match ev {
            Incoming::Status(s) => self.status = s,
            Incoming::Control(c) => match c {
                ControlCmd::ClickThrough(v) => self.cfg.window.click_through = v.unwrap_or(!self.cfg.window.click_through),
                ControlCmd::Background(b) => self.cfg.window.background = b,
                ControlCmd::ClearQueue => {
                    self.queue.clear();
                    self.cats.clear();
                }
                ControlCmd::Running(r) => {
                    self.running = r;
                    self.reconnect();
                }
                ControlCmd::Layout(l) => self.cfg.window.layout = if l == "full" { "full".into() } else { "top".into() },
                ControlCmd::Theme(m) => {
                    self.sync_edit_buffers();
                    self.cfg.theme.mode = Theme::from_key(&m).key().into();
                    self.load_gift_rows();
                }
                ControlCmd::Save => self.save_config(ctx),
                ControlCmd::Reload => self.reload_config(),
                ControlCmd::Idle(m) => {
                    if ["still", "ambient", "hidden"].contains(&m.as_str()) {
                        self.cfg.window.idle = m;
                    }
                }
                ControlCmd::Update(u) => match u.as_str() {
                    "apply" => self.updater.apply(),
                    "bridge" => self.updater.update_bridge(self.cfg.tiktok.node_command.clone()),
                    _ => self.updater.check(),
                },
            },
            Incoming::Direct(DirectIn { action, user, diamonds, .. }) => {
                self.gifts_received += 1;
                if self.running {
                    self.enqueue(action, &user, diamonds, 99)
                }
            }
            Incoming::Gift(GiftIn { user, gift_id, gift_name, diamonds, repeat }) => {
                self.gifts_received += 1;
                if !self.running {
                    return;
                }
                let act = self.cfg.action_for_gift(&gift_name, gift_id);
                let Some(action) = Action::parse(&act) else {
                    eprintln!("[gift] {user} ส่ง {gift_name} ({gift_id}) -> ไม่ได้ตั้งค่า (none)");
                    return;
                };
                let total = diamonds.max(1) * repeat.max(1) as u64;
                let plays = repeat.clamp(1, self.cfg.tiktok.max_plays_per_gift.max(1));
                eprintln!("[gift] {user} ส่ง {gift_name} x{repeat} ({total} เพชร) -> {act}");
                for _ in 0..plays {
                    self.enqueue(action, &user, total, 99);
                }
            }
        }
    }

    fn reload_config(&mut self) {
        self.cfg = Config::load(&self.cfg_path);
        self.load_gift_rows();
        self.fortunes_text = self.cfg.siamsi.fortunes.join("\n");
        self.blessings_text = self.cfg.krathong.blessings.join("\n");
        self.thanks_text = self.cfg.cat.thanks.join("\n");
        self.flash("โหลดการตั้งค่าใหม่แล้ว".into());
    }

    /// timeline (seconds of show-time): walk-in, act, reveal, walk-out
    fn durations(&self, s: &Show) -> (f32, f32, f32, f32) {
        let reveal = self.cfg.timing.reveal_seconds.max(1.5) + s.tier as f32 * 0.5;
        match (s.theme, s.action) {
            (Theme::Shrine, Action::Pray) => (WALK, 3.4, reveal, OUT),
            (Theme::Shrine, Action::Siamsi) => (WALK, 6.4, reveal + 0.6, OUT),
            (Theme::Krathong, Action::Pray) => (WALK, kr_push_end(Action::Pray) + 0.5, reveal, OUT),
            (Theme::Krathong, Action::Siamsi) => (WALK, kr_push_end(Action::Siamsi) + 0.5, reveal + 0.6, OUT),
            (Theme::Cat, _) => (cat::WALK_MAX, cat::POUR + cat::EAT, reveal, cat::LEAVE),
        }
    }
    fn target_x(th: Theme) -> f32 {
        match th {
            Theme::Shrine => PRAY_X,
            Theme::Krathong => kr::KNEEL_X,
            Theme::Cat => cat::stand_x(),
        }
    }
    fn char_scale(th: Theme) -> f32 {
        match th {
            Theme::Shrine => 1.2,
            Theme::Krathong => 1.1,
            Theme::Cat => 1.0,
        }
    }

    fn active(&self) -> bool {
        self.current.is_some() || !self.queue.is_empty() || self.cats.busy() || self.gap > 0.0 || self.particles.iter().any(|p| !matches!(p.kind, 3 | 4 | 8))
    }

    // ------------------------------------------------------------------ simulation
    fn update(&mut self, dt_real: f32) {
        let speed = self.cfg.timing.speed.clamp(0.25, 4.0);
        let dt = dt_real * speed;
        self.time += dt_real;
        let active = self.active();
        if active {
            self.idle_for = 0.0;
        } else {
            self.idle_for += dt_real;
        }
        let idle_mode = self.cfg.window.idle.clone();
        let freeze = !active && idle_mode != "ambient" && self.idle_for >= 1.2;
        if !freeze {
            self.scene_time += dt;
        }
        // hidden idle: fade whole world out / in
        let target_alpha = if idle_mode == "hidden" && !active { 0.0 } else { 1.0 };
        let rate = if target_alpha > self.scene_alpha { 3.0 } else { 1.2 };
        self.scene_alpha += (target_alpha - self.scene_alpha).clamp(-rate * dt_real, rate * dt_real);
        self.shake *= (-6.0 * dt_real).exp();

        // queue / transitions
        if self.current.is_none() {
            if self.gap > 0.0 {
                self.gap -= dt;
            } else if !self.queue.is_empty() && (idle_mode != "hidden" || self.scene_alpha > 0.95) {
                let mut s = self.queue.pop_front().unwrap();
                s.start_x = s.qx.unwrap_or(-150.0);
                self.shows_started += 1;
                self.current = Some(s);
                self.entry_poof();
            }
        }

        if let Some(mut s) = self.current.take() {
            let (w, a, r, o) = self.durations(&s);
            s.t += dt;
            self.step_show(&mut s, dt);
            if s.t >= w + a + r + o {
                self.shows_finished += 1;
                self.gap = 0.7;
            } else {
                self.current = Some(s);
            }
        }

        self.step_line(dt);
        let theme = self.theme();
        if theme == Theme::Cat {
            let before_s = self.cats.started;
            let before_f = self.cats.finished;
            self.cats.tick(dt, self.cfg.cat.max_visible, self.cfg.timing.reveal_seconds, self.vh());
            self.shows_started += self.cats.started.saturating_sub(before_s);
            self.shows_finished += self.cats.finished.saturating_sub(before_f);
        }
        if active || idle_mode == "ambient" {
            self.spawn_ambient(dt, theme);
        } else if idle_mode == "still" {
            // ambient motes/petals fade away so the idle frame is clean
            for p in &mut self.particles {
                if matches!(p.kind, 3 | 4) {
                    p.life = p.life.min(0.5);
                }
            }
            self.cats.fade_ambient();
        }
        if !freeze {
            self.step_particles(dt);
            let max_f = self.cfg.krathong.max_floating.max(1);
            let life = self.cfg.krathong.float_seconds.max(10.0);
            for f in &mut self.floating {
                f.age += dt;
                f.vx = (f.vx * (-0.6 * dt).exp()).max(7.0);
                f.x += f.vx * dt;
                if f.fade.is_none() && (f.age > life || f.x > VW - 50.0) {
                    f.fade = Some(0.0);
                }
                if let Some(fd) = &mut f.fade {
                    *fd += dt;
                }
            }
            let live = self.floating.iter().filter(|f| f.fade.is_none()).count();
            if live > max_f {
                let mut extra = live - max_f;
                for f in self.floating.iter_mut() {
                    if extra == 0 {
                        break;
                    }
                    if f.fade.is_none() {
                        f.fade = Some(0.0);
                        extra -= 1;
                    }
                }
            }
            self.floating.retain(|f| f.fade.map(|x| x < 2.0).unwrap_or(true));
        }
        if self.flash_msg.1 > 0.0 {
            self.flash_msg.1 -= dt_real;
        }
    }

    fn entry_poof(&mut self) {
        let Some(s) = self.current.as_ref() else { return };
        let g = self.ground_of(s.theme);
        let tier = s.tier;
        for _ in 0..(10 + tier * 8) {
            let a = self.rng.f32() * std::f32::consts::TAU;
            let mut p = Particle::new(1, Pos2::new(10.0, g - 60.0 - self.rng.f32() * 160.0), Vec2::new(a.cos().abs() * 140.0, a.sin() * 80.0), 1.0, if tier >= 2 { GOLD } else { Color32::WHITE }, 3.5);
            p.drag = 2.0;
            p.rot = self.rng.f32() * 10.0;
            self.particles.push(p);
        }
    }

    fn step_show(&mut self, s: &mut Show, dt: f32) {
        let (w, a_d, _r, _o) = self.durations(s);
        let t = s.t;
        let a = t - w;
        let rt = a - a_d;
        let g = self.ground_of(s.theme);
        // fireworks schedule (relative to reveal)
        while s.fired < s.bursts.len() && rt >= s.bursts[s.fired].0 {
            let kind = s.bursts[s.fired].1;
            self.spawn_effect(kind, s.tier, s.theme);
            s.fired += 1;
        }
        // gold trail while walking (tier 3+)
        if s.tier >= 2 && t < w {
            s.trail_acc += dt * (s.tier as f32 * 12.0);
            while s.trail_acc >= 1.0 {
                s.trail_acc -= 1.0;
                let x = self.walk_x(s);
                let c = if self.rng.bool() { GOLD } else { Color32::WHITE };
                let mut p = Particle::new(1, Pos2::new(x - 20.0 - self.rng.f32() * 30.0, g - 10.0 - self.rng.f32() * 180.0), Vec2::new(-20.0, -25.0), 1.1, c, 3.5);
                p.rot = self.rng.f32() * 10.0;
                self.particles.push(p);
            }
        }
        // top tier: golden sparkle rain
        if s.tier >= 4 && self.rng.f32() < 30.0 * dt {
            let mut p = Particle::new(1, Pos2::new(self.rng.f32() * VW, -10.0), Vec2::new(0.0, 110.0 + self.rng.f32() * 80.0), 5.0, GOLD, 3.5);
            p.rot = self.rng.f32() * 10.0;
            self.particles.push(p);
        }
        // number reveal: per-digit stop events -> sparkle burst + flash
        if s.action == Action::Pray && rt >= 0.0 {
            let geo = self.pray_geo(s);
            let mut hits = vec![];
            for (i, row) in s.digit_done.iter_mut().enumerate() {
                for (j, done) in row.iter_mut().enumerate() {
                    if !*done && rt >= digit_stop(i, j) {
                        *done = true;
                        hits.push((geo.digit_x[i][j], geo.row_y[i]));
                    }
                }
            }
            for (x, y) in hits {
                let mut fl = Particle::new(7, Pos2::new(x, y), Vec2::ZERO, 0.35, Color32::from_rgb(255, 240, 180), 90.0);
                fl.max = 0.35;
                self.particles.push(fl);
                for _ in 0..(12 + s.tier * 4) {
                    let ang = self.rng.f32() * std::f32::consts::TAU;
                    let sp = 120.0 + self.rng.f32() * 160.0;
                    let mut p = Particle::new(1, Pos2::new(x, y), Vec2::new(ang.cos(), ang.sin()) * sp, 0.9, if self.rng.bool() { GOLD } else { Color32::WHITE }, 3.0 + s.tier as f32 * 0.5);
                    p.drag = 3.0;
                    p.grav = 60.0;
                    p.rot = self.rng.f32() * 10.0;
                    self.particles.push(p);
                }
            }
        }
        match s.theme {
            Theme::Shrine => {
                if s.action == Action::Siamsi {
                    self.step_stick(s, dt, g);
                }
            }
            Theme::Cat => {}
            Theme::Krathong => {
                let push_end = kr_push_end(s.action);
                if !s.released && a >= push_end {
                    s.released = true;
                    let x0 = kr::BANK_EDGE + 110.0;
                    self.floating.push(Floating { x: x0, y: s.lane_y, vx: 70.0, age: 0.0, fade: None, tier: s.tier, name: s.user.clone(), seed: s.look.seed });
                    for _ in 0..14 {
                        let ang = -self.rng.f32() * std::f32::consts::PI;
                        let mut p = Particle::new(5, Pos2::new(x0, s.lane_y), Vec2::new(ang.cos() * 90.0, ang.sin() * 70.0), 0.6, Color32::from_rgb(190, 220, 255), 3.0);
                        p.grav = 300.0;
                        self.particles.push(p);
                    }
                    if s.tier >= 2 {
                        let n = [0, 0, 2, 4, 9][s.tier.min(4)];
                        for i in 0..n {
                            let mut p = Particle::new(8, Pos2::new(kr::KNEEL_X + 60.0 + i as f32 * 26.0 - n as f32 * 8.0, g - 150.0 - self.rng.f32() * 40.0), Vec2::new(self.rng.f32() * 24.0 - 6.0, -(30.0 + self.rng.f32() * 24.0)), 18.0, GOLD, 1.0 + self.rng.f32() * 0.35);
                            p.rot = self.rng.f32() * 10.0;
                            self.particles.push(p);
                        }
                    }
                }
                // wishing sparkles rising from the joined hands
                let (ws, we) = kr_wish_window(s.action);
                if a > ws + 0.4 && a < we {
                    s.wish_acc += dt * if s.action == Action::Siamsi { 26.0 } else { 12.0 };
                    while s.wish_acc >= 1.0 {
                        s.wish_acc -= 1.0;
                        let (x, pose) = self.animate(s);
                        let rig = solve_rig(x, g, &pose, Self::char_scale(s.theme));
                        let hd = rig.arms[1].1;
                        let mut p = Particle::new(1, Pos2::new(hd.0, hd.1 - 20.0), Vec2::new(self.rng.f32() * 40.0 - 20.0, -(60.0 + self.rng.f32() * 60.0)), 1.6, if self.rng.bool() { GOLD } else { Color32::from_rgb(255, 210, 235) }, 3.5);
                        p.rot = self.rng.f32() * 10.0;
                        self.particles.push(p);
                    }
                }
            }
        }
    }

    fn step_stick(&mut self, s: &mut Show, dt: f32, g: f32) {
        let a = s.t - WALK;
        if s.stick.is_none() && a >= POP {
            let (x, pose) = self.animate(s);
            let k = Self::char_scale(s.theme);
            let rig = solve_rig(x, g, &pose, k);
            let hd = rig.arms[1].1;
            let f = pose.facing;
            let sc = 0.75 * k;
            let ang = 0.35 * f;
            let base = (hd.0 + 6.0 * k * f, hd.1 + 30.0 * k);
            let (sn, cs) = ang.sin_cos();
            let ly = -230.0 * sc;
            let pos = (base.0 - ly * sn, base.1 + ly * cs);
            s.stick = Some(StickPhys { x: pos.0, y: pos.1, vx: 105.0 * f, vy: -520.0, ang: -std::f32::consts::FRAC_PI_2 + ang, va: 9.0 * f, landed: None });
        }
        let mut dust = None;
        if let Some(st) = &mut s.stick {
            if st.landed.is_none() {
                st.vy += 1500.0 * dt;
                st.x += st.vx * dt;
                st.y += st.vy * dt;
                st.ang += st.va * dt;
                let floor = g - 8.0;
                if st.y >= floor {
                    st.y = floor;
                    if st.vy > 140.0 {
                        st.vy = -st.vy * 0.42;
                        st.vx *= 0.6;
                        st.va *= -0.5;
                    } else {
                        st.vy = 0.0;
                        st.landed = Some(a);
                    }
                    dust = Some((st.x, floor));
                }
            } else {
                st.vx *= (-6.0 * dt).exp();
                st.x += st.vx * dt;
                // settle lying flat on the floor
                let half = 0.0;
                let target = ((st.ang - half) / std::f32::consts::PI).round() * std::f32::consts::PI + half;
                st.ang += (target - st.ang) * (1.0 - (-10.0 * dt).exp());
            }
        }
        if let Some((x, y)) = dust {
            for _ in 0..8 {
                let ang = -self.rng.f32() * std::f32::consts::PI;
                let mut p = Particle::new(5, Pos2::new(x, y), Vec2::new(ang.cos() * 70.0, ang.sin() * 40.0), 0.5, Color32::from_rgb(230, 210, 170), 4.0);
                p.drag = 3.0;
                self.particles.push(p);
            }
        }
    }

    /// x of waiting-line slot k (0 = right behind the one at the shrine); None if it would be off-screen
    fn line_slot(&self, th: Theme, k: usize) -> Option<f32> {
        let tx = Self::target_x(th);
        let big = self.current.as_ref().map(|c| c.tier >= 4 && c.theme == th).unwrap_or(false);
        let (first, gap, min_x) = match th {
            Theme::Shrine => (tx - if big { 330.0 } else { 180.0 }, 145.0, 60.0),
            Theme::Krathong => (tx - if big { 250.0 } else { 125.0 }, 118.0, 45.0),
            Theme::Cat => (tx - if big { 200.0 } else { 160.0 }, 160.0, 50.0),
        };
        let x = first - gap * k as f32;
        (x >= min_x).then_some(x)
    }

    /// the visible queue: the next senders walk out and stand in a single-file line behind the active one
    fn step_line(&mut self, dt: f32) {
        let th = self.theme();
        let max_vis = self.cfg.window.max_visible.clamp(1, 12);
        let room = max_vis.saturating_sub(if self.current.is_some() { 1 } else { 0 });
        let slots: Vec<Option<f32>> = (0..self.queue.len()).map(|k| if k < room { self.line_slot(th, k) } else { None }).collect();
        let mut prev_x = f32::INFINITY;
        for (k, s) in self.queue.iter_mut().enumerate() {
            let Some(target) = slots[k] else {
                // not enough room: anyone already out keeps their place, the rest wait off-screen
                if let Some(x) = s.qx { prev_x = x; }
                continue;
            };
            let x = s.qx.get_or_insert(-150.0 - 40.0 * k as f32);
            s.qage += dt;
            // walk toward the slot, never into the person ahead
            let goal = target.min(prev_x - 130.0);
            let step = (goal - *x).clamp(-240.0 * dt, 240.0 * dt);
            *x += step;
            let moving = step.abs() > 0.5 * 240.0 * dt;
            s.qmove += ((if moving { 1.0 } else { 0.0 }) - s.qmove) * (1.0 - (-10.0 * dt).exp());
            s.qphase += step.abs() * std::f32::consts::TAU / 95.0;
            prev_x = *x;
        }
    }

    fn draw_line(&self, cx: &Cx, sprites: &Sprites) {
        let th = self.theme();
        let k = Self::char_scale(th);
        // back of the line first so nearer ones overlap correctly
        let mut v: Vec<(usize, &Show)> = self.queue.iter().enumerate().filter(|(_, s)| s.qx.map(|x| x > -120.0).unwrap_or(false)).collect();
        v.reverse();
        for (idx, s) in v {
            let x = s.qx.unwrap();
            let mut p = Pose::default();
            let st = self.scene_time;
            p.breath = (st * 2.4 + s.look.seed).sin();
            p.blink = blink(st, s.look.seed);
            p.walk = s.qmove.clamp(0.0, 1.0);
            p.phase = s.qphase;
            p.squash = -0.6 * p.walk * (2.0 * p.phase).sin().abs() + 0.25 * p.walk;
            p.cape_lag = 18.0 * p.walk;
            p.expr = if s.qmove < 0.2 && (st * 0.7 + s.look.seed).sin() > 0.6 { 1 } else { 0 };
            if th == Theme::Krathong {
                p.hands = 1.0;
                p.carry = 1.0;
            }
            let look = Look { tier: s.tier, shirt: s.look.shirt, pants: s.look.pants, scale: k, hair: s.look.hair, style: s.look.style };
            let rig = draw_character(cx, sprites, x, &p, &look);
            if th == Theme::Krathong {
                let hd = rig.arms[1].1;
                kr::draw_krathong(cx, hd.0 + 6.0, hd.1 + 4.0, 0.62, s.tier, s.look.seed, None, 1.0);
            }
            let head_top = rig.head.1 - HEAD_R * k * 1.05 - if s.tier >= 3 { 70.0 * k } else { 8.0 };
            let lift = if idx % 2 == 1 { 44.0 } else { 0.0 };
            if lift > 0.0 {
                cx.seg((x, head_top + 2.0), (x, head_top - lift), 2.0, rgba(255, 255, 255, 0.7));
            }
            let short = truncate(&s.user, 9);
            draw_name_banner(cx, x, head_top - lift, &short, s.tier.min(2), "", s.qage / 0.9, 1.0);
            tag_pointer(cx, x, head_top - lift, s.look.tag, 1.0);
        }
    }

    fn walk_x(&self, s: &Show) -> f32 {
        lerp(s.start_x, Self::target_x(s.theme), smooth(s.t / WALK))
    }

    /// Character position + pose for the current show time.
    fn animate(&self, s: &Show) -> (f32, Pose) {
        let (w, a_d, r, o) = self.durations(s);
        let t = s.t;
        let st = self.scene_time;
        let tx = Self::target_x(s.theme);
        let mut p = Pose::default();
        p.breath = (st * 2.4 + s.look.seed).sin();
        p.blink = blink(st, s.look.seed);
        let x;
        if t < w {
            let u = t / w;
            x = lerp(s.start_x, tx, smooth(u));
            let spd = 6.0 * u * (1.0 - u) * ((tx - s.start_x) / (tx + 150.0)).clamp(0.25, 1.0).sqrt();
            p.walk = (spd * 1.25).clamp(0.0, 1.0);
            p.phase = (x - s.start_x) * std::f32::consts::TAU / 95.0 + s.qphase;
            p.squash = -0.6 * p.walk * (2.0 * p.phase).sin().abs() + 0.25 * p.walk;
            p.cape_lag = 18.0 * p.walk;
            if s.theme == Theme::Krathong {
                p.hands = 1.0;
                p.carry = 1.0;
            }
        } else if t < w + a_d + r {
            x = tx;
            let a = t - w;
            // anticipation (small rise) then kneel with overshoot + landing squash
            p.kneel = ease_out_back((a - 0.1) / 0.55);
            p.squash = if a < 0.15 { -0.6 * (a / 0.15 * std::f32::consts::PI).sin() } else if a > 0.5 && a < 0.85 { 0.8 * ((a - 0.5) / 0.35 * std::f32::consts::PI).sin() } else { 0.0 };
            p.cape_lag = 18.0 * (1.0 - smooth(a / 0.8)) * (a * 9.0).cos();
            match (s.theme, s.action) {
                (Theme::Shrine, Action::Pray) => {
                    p.hands = smooth((a - 0.45) / 0.45);
                    p.raise = ease_in_out((a - 0.95) / 0.5) * (1.0 - 0.7 * smooth((a - 3.0) / 0.4));
                    p.bow = ease_in_out((a - 1.5) / 0.6) * (1.0 - ease_in_out((a - 2.5) / 0.5));
                    p.expr = if a > 0.9 { 1 } else { 0 };
                    if a > a_d {
                        p.raise = 0.3;
                        p.expr = 2;
                    }
                }
                (Theme::Shrine, Action::Siamsi) => {
                    p.hands = smooth((a - 0.3) / 0.4);
                    let env = smooth((a - 0.7) / 0.3) * (1.0 - smooth((a - 3.2) / 0.3));
                    let tilt = ((a - 0.7) * std::f32::consts::TAU * 3.0).sin() * 0.24 * env;
                    let lift = if a < 2.7 { -1.0 } else if a < POP { smooth((a - 2.7) / (POP - 2.7)) } else { -2.0 };
                    p.hold_cyl = Some((tilt, if env > 0.01 { a } else { 0.0 }, lift));
                    p.bow = 0.12 + 0.1 * env * ((a - 0.7) * std::f32::consts::TAU * 3.0).cos().abs();
                    p.expr = if a < POP { 1 } else { 2 };
                }
                (Theme::Cat, _) => {}
                (Theme::Krathong, act) => {
                    let (ws, we) = kr_wish_window(act);
                    let pe = kr_push_end(act);
                    p.hands = 1.0;
                    // carry -> place -> wai/wish -> reach -> push -> chest
                    if a < 0.6 {
                        p.carry = 1.0;
                    } else if a < 1.1 {
                        p.carry = 1.0;
                        p.reach = Some((self.kr_bank_spot(), smooth((a - 0.6) / 0.5)));
                    } else if a < ws {
                        p.reach = Some((self.kr_bank_spot(), 1.0 - smooth((a - 1.1) / 0.3)));
                    } else if a < we {
                        p.raise = ease_in_out((a - ws) / 0.5);
                        p.bow = 0.6 * ease_in_out((a - ws - 0.4) / 0.4) * (1.0 - ease_in_out((a - we + 0.5) / 0.4));
                        p.expr = 1;
                    } else if a < pe {
                        let u = (a - we) / (pe - we);
                        let kp = self.kr_pos(s, a);
                        p.reach = Some((kp, smooth(u * 3.0) * (1.0 - smooth((u - 0.7) / 0.3))));
                    } else {
                        p.raise = 0.2;
                        p.expr = 2;
                    }
                }
            }
        } else {
            let ot = t - w - a_d - r;
            p.kneel = 1.0 - ease_out_cubic(ot / 0.35);
            p.facing = if ot > 0.18 { -1.0 } else { 1.0 };
            let u = ((ot - 0.3) / (o - 0.3)).clamp(0.0, 1.0);
            let eu = u * u * (1.5 - 0.5 * u);
            x = lerp(tx, -170.0, eu);
            p.walk = smooth(u * 4.0);
            p.phase = (tx - x) * std::f32::consts::TAU / 95.0;
            p.squash = -0.6 * p.walk * (2.0 * p.phase).sin().abs() + 0.25 * p.walk;
            p.cape_lag = 18.0 * p.walk;
            p.expr = if ot < 0.3 { 2 } else { 0 };
        }
        (x, p)
    }

    fn kr_bank_spot(&self) -> (f32, f32) {
        (kr::KNEEL_X + 80.0, kr::bank_y(self.vh()) - 4.0)
    }

    /// krathong position while it is still handled by the character
    fn kr_pos(&self, s: &Show, a: f32) -> (f32, f32) {
        let bank = self.kr_bank_spot();
        let (_ws, we) = kr_wish_window(s.action);
        let pe = kr_push_end(s.action);
        if a < we {
            return bank;
        }
        let u = smooth((a - we - 0.25) / (pe - we - 0.25));
        let end = (kr::BANK_EDGE + 110.0, s.lane_y);
        let x = lerp(bank.0, end.0, u);
        let y = lerp(bank.1, end.1, u) - (u * std::f32::consts::PI).sin() * 30.0;
        (x, y)
    }

    fn step_particles(&mut self, dt: f32) {
        let mut explode: Vec<(Pos2, usize)> = vec![];
        let mut crackles: Vec<Pos2> = vec![];
        let mut trails: Vec<Pos2> = vec![];
        let time = self.time;
        for p in &mut self.particles {
            p.life -= dt;
            match p.kind {
                2 => {
                    p.vel.y += 600.0 * dt;
                    trails.push(p.pos);
                    if p.vel.y > -30.0 {
                        p.life = 0.0;
                        explode.push((p.pos, p.tier));
                    }
                }
                3 => {
                    p.vel.x += (time * 1.7 + p.rot).sin() * 30.0 * dt;
                    p.rot += p.vrot * dt;
                }
                8 => {
                    p.vel.x += (time * 0.5 + p.rot).sin() * 6.0 * dt;
                }
                _ => {}
            }
            p.vel.y += p.grav * dt;
            if p.drag > 0.0 {
                p.vel *= (-p.drag * dt).exp();
            }
            p.pos += p.vel * dt;
            if p.life <= 0.0 && p.crackle {
                crackles.push(p.pos);
            }
        }
        self.particles.retain(|p| p.life > 0.0 && p.pos.y < 4000.0 && p.pos.y > -400.0);
        for pos in trails {
            let mut e = Particle::new(5, pos, Vec2::new(self.rng.f32() * 20.0 - 10.0, 20.0), 0.45, Color32::from_rgb(255, 190, 90), 3.0);
            e.grav = 40.0;
            self.particles.push(e);
        }
        for pos in crackles {
            for _ in 0..4 {
                let a = self.rng.f32() * std::f32::consts::TAU;
                let mut p = Particle::new(0, pos, Vec2::new(a.cos(), a.sin()) * 60.0, 0.25, Color32::WHITE, 1.6);
                p.drag = 2.0;
                self.particles.push(p);
            }
        }
        for (pos, tier) in explode {
            self.explode(pos, tier);
        }
        if self.particles.len() > 6000 {
            let n = self.particles.len() - 6000;
            self.particles.drain(0..n);
        }
    }

    fn spawn_ambient(&mut self, dt: f32, theme: Theme) {
        let vh = self.vh();
        let busy = self.current.as_ref().map(|s| s.tier).unwrap_or(0);
        match theme {
            Theme::Shrine => {
                let g = ground_for(vh);
                if self.rng.f32() < 3.0 * dt {
                    let mut p = Particle::new(4, Pos2::new(SHRINE_X + (self.rng.f32() - 0.5) * 420.0, g - 40.0 - self.rng.f32() * 340.0), Vec2::new(self.rng.f32() * 10.0 - 5.0, -(6.0 + self.rng.f32() * 12.0)), 4.0 + self.rng.f32() * 3.0, Color32::from_rgb(255, 230, 160), 3.0);
                    p.rot = self.rng.f32() * 10.0;
                    self.particles.push(p);
                }
                let petal_rate = if busy >= 2 { 2.5 } else { 0.5 };
                if self.rng.f32() < petal_rate * dt {
                    let c = if self.rng.bool() { Color32::from_rgb(255, 182, 200) } else { Color32::from_rgb(255, 240, 245) };
                    let mut p = Particle::new(3, Pos2::new(self.rng.f32() * VW, -20.0), Vec2::new(20.0 + self.rng.f32() * 25.0, 40.0 + self.rng.f32() * 30.0), (g + 20.0) / 50.0, c, 7.0 + self.rng.f32() * 3.0);
                    p.rot = self.rng.f32() * 6.0;
                    p.vrot = 1.5 + self.rng.f32() * 2.0;
                    self.particles.push(p);
                }
            }
            Theme::Krathong => {
                let wt = kr::water_top(vh);
                if self.rng.f32() < 0.35 * dt {
                    let mut p = Particle::new(8, Pos2::new(500.0 + self.rng.f32() * 560.0, wt - 10.0), Vec2::new(self.rng.f32() * 8.0 - 4.0, -(10.0 + self.rng.f32() * 8.0)), 26.0, GOLD, 0.35 + self.rng.f32() * 0.25);
                    p.rot = self.rng.f32() * 10.0;
                    self.particles.push(p);
                }
            }
            Theme::Cat => {
                self.cats.spawn_ambient(dt, busy > 0 || self.cats.busy());
            }
        }
    }

    fn spawn_effect(&mut self, kind: u8, tier: usize, theme: Theme) {
        let vh = self.vh();
        let launch_y = match theme {
            Theme::Shrine => ground_for(vh),
            Theme::Krathong => kr::water_top(vh),
            Theme::Cat => cat::floor_y(vh),
        };
        match kind {
            0 => {
                let cxx = match theme {
                    Theme::Krathong => 650.0,
                    Theme::Cat => 540.0,
                    Theme::Shrine => 430.0,
                };
                for _ in 0..(30 + tier * 20) {
                    let a = self.rng.f32() * std::f32::consts::TAU;
                    let sp = 80.0 + self.rng.f32() * (120.0 + tier as f32 * 60.0);
                    let c = pick_color(&mut self.rng, tier);
                    let mut p = Particle::new(1, Pos2::new(cxx, 200.0), Vec2::new(a.cos(), a.sin()) * sp, 1.6, c, 4.0 + tier as f32);
                    p.drag = 1.5;
                    p.rot = self.rng.f32() * 10.0;
                    self.particles.push(p);
                }
            }
            1 => {
                // rocket: apex inside the upper area (keeps fireworks off the streamer's face)
                let top_area = if self.cfg.window.layout == "full" { vh * 0.45 } else { vh };
                let ty = top_area * (0.08 + self.rng.f32() * 0.32);
                let h = (launch_y - ty).max(120.0);
                let vy = -(2.0 * 600.0 * h).sqrt();
                let x = 80.0 + self.rng.f32() * (VW - 160.0);
                let mut p = Particle::new(2, Pos2::new(x, launch_y), Vec2::new(self.rng.f32() * 60.0 - 30.0, vy), 6.0, GOLD, 4.0);
                p.tier = tier;
                self.particles.push(p);
            }
            _ => {
                for _ in 0..(60 + tier * 30) {
                    let x = self.rng.f32() * VW;
                    let mut p = Particle::new(1, Pos2::new(x, -self.rng.f32() * 200.0), Vec2::new(self.rng.f32() * 30.0 - 15.0, 160.0 + self.rng.f32() * 160.0), 3.5, if self.rng.bool() { GOLD } else { Color32::from_rgb(255, 240, 200) }, 4.0);
                    p.rot = self.rng.f32() * 10.0;
                    self.particles.push(p);
                }
            }
        }
    }

    fn explode(&mut self, pos: Pos2, tier: usize) {
        let t = tier.min(4);
        let shapes: &[u8] = match t {
            0 | 1 => &[0],
            2 => &[0, 1],
            3 => &[0, 1, 2, 3],
            _ => &[0, 1, 2, 3, 4],
        };
        let shape = shapes[self.rng.usize(0..shapes.len())];
        let n = [50, 70, 90, 130, 180][t];
        let sp = [150.0, 180.0, 220.0, 270.0, 330.0][t];
        let c1 = pick_color(&mut self.rng, t);
        let c2 = pick_color(&mut self.rng, t);
        let crackle = t >= 4 || (t >= 3 && self.rng.bool());
        let push = |rng: &mut fastrand::Rng, ps: &mut Vec<Particle>, v: Vec2, c: Color32, kind: u8, life: f32, drag: f32, grav: f32, size: f32| {
            let mut p = Particle::new(kind, pos, v, life * (0.85 + rng.f32() * 0.3), c, size);
            p.drag = drag;
            p.grav = grav;
            p.rot = rng.f32() * 10.0;
            p.crackle = crackle && rng.f32() < 0.35;
            p.tier = t;
            ps.push(p);
        };
        let tau = std::f32::consts::TAU;
        match shape {
            1 => {
                for i in 0..n {
                    let a = i as f32 / n as f32 * tau;
                    push(&mut self.rng, &mut self.particles, Vec2::new(a.cos(), a.sin()) * sp, c1, 0, 1.6, 1.4, 110.0, 3.4);
                }
                for _ in 0..n / 3 {
                    let a = self.rng.f32() * tau;
                    let r = sp * 0.3 * self.rng.f32();
                    push(&mut self.rng, &mut self.particles, Vec2::new(a.cos(), a.sin()) * r, c2, 1, 1.2, 1.5, 40.0, 3.0);
                }
            }
            2 => {
                for i in 0..n {
                    let a = i as f32 / n as f32 * tau;
                    let hx = 16.0 * a.sin().powi(3);
                    let hy = -(13.0 * a.cos() - 5.0 * (2.0 * a).cos() - 2.0 * (3.0 * a).cos() - (4.0 * a).cos());
                    push(&mut self.rng, &mut self.particles, Vec2::new(hx, hy) / 16.0 * sp, Color32::from_rgb(255, 100, 150), 0, 1.7, 1.6, 50.0, 3.6);
                }
            }
            3 => {
                for _ in 0..n {
                    let a = self.rng.f32() * tau;
                    let v = Vec2::new(a.cos(), a.sin()) * sp * (0.6 + self.rng.f32() * 0.4);
                    push(&mut self.rng, &mut self.particles, v, GOLD, 6, 2.9, 1.9, 90.0, 3.0);
                }
            }
            4 => {
                for i in 0..n {
                    let a = i as f32 / n as f32 * tau;
                    push(&mut self.rng, &mut self.particles, Vec2::new(a.cos(), a.sin()) * sp, c1, 0, 1.7, 1.3, 100.0, 3.6);
                    push(&mut self.rng, &mut self.particles, Vec2::new(a.cos(), a.sin()) * sp * 0.55, c2, 0, 1.5, 1.3, 100.0, 3.0);
                }
            }
            _ => {
                for _ in 0..n {
                    let a = self.rng.f32() * tau;
                    let v = Vec2::new(a.cos(), a.sin()) * sp * (0.55 + self.rng.f32() * 0.45);
                    let c = if self.rng.f32() < 0.7 { c1 } else { c2 };
                    push(&mut self.rng, &mut self.particles, v, c, 0, 1.5, 1.2, 140.0, 3.0 + t as f32 * 0.4);
                }
            }
        }
        let mut fl = Particle::new(7, pos, Vec2::ZERO, 0.3, Color32::from_rgb(255, 235, 190), 70.0 + t as f32 * 25.0);
        fl.max = 0.3;
        self.particles.push(fl);
        if t >= 3 {
            self.shake = self.shake.max(if t >= 4 { 9.0 } else { 5.0 });
        }
    }

    // ------------------------------------------------------------------ drawing
    fn draw(&mut self, ui: &mut egui::Ui) {
        let rect = ui.ctx().content_rect();
        let mut painter = ui.painter_at(rect);
        let bg = self.cfg.window.background.clone();
        if bg == "green" {
            painter.rect_filled(rect, 0, Color32::from_rgb(0, 255, 0));
        }
        if self.cfg.window.idle == "hidden" {
            if self.scene_alpha <= 0.002 {
                return;
            }
            painter.multiply_opacity(self.scene_alpha);
        }
        let theme = self.theme();
        let mut cx = Cx::new(&painter, rect, self.scene_time);
        cx.soft = bg != "green";
        if theme == Theme::Krathong {
            cx.gy = Some(kr::bank_y(cx.vh));
        } else if theme == Theme::Cat {
            cx.gy = Some(cat::floor_y(cx.vh));
        }
        // camera: zoom on the landed เซียมซี stick + screen shake
        if let Some(s) = &self.current {
            if let Some(st) = &s.stick {
                if let Some(l) = st.landed {
                    let zt = s.t - WALK - l;
                    let z = if zt < 1.3 { ease_in_out((zt - 0.15) / 0.5) } else { 1.0 - ease_in_out((zt - 1.3) / 0.45) };
                    cx.z = 1.0 + 0.85 * z;
                    cx.fx = st.x;
                    cx.fy = st.y - 40.0;
                    cx.ox = (VW * 0.5 - st.x) * 0.35 * z;
                    cx.oy = (cx.vh * 0.55 - st.y) * 0.35 * z;
                }
            }
        }
        if self.shake > 0.05 {
            cx.ox += noise1(self.time * 40.0) * self.shake;
            cx.oy += noise1(self.time * 37.0 + 5.0) * self.shake;
        }
        let sprites = self.sprites.as_ref().unwrap();
        match theme {
            Theme::Shrine => self.draw_shrine_world(&cx, sprites, &bg),
            Theme::Krathong => self.draw_krathong_world(&cx, sprites, &bg),
            Theme::Cat => self.cats.draw_world(&cx, &bg, self.cfg.timing.reveal_seconds, self.cfg.cat.max_visible),
        }
        draw_particles(&cx, &self.particles, false);

        // screen-space layer (no camera)
        let mut ui_cx = Cx::new(&painter, rect, self.scene_time);
        ui_cx.soft = bg != "green";
        if let Some(s) = &self.current {
            if s.tier >= 4 {
                let (w, a, r, _) = self.durations(s);
                let fade = smooth(s.t / 0.6) * (1.0 - smooth((s.t - (w + a + r)) / 0.8));
                shimmer(&ui_cx, fade);
            }
            self.draw_reveal(&ui_cx, s);
        }
        if theme == Theme::Cat {
            if self.cats.shows().next().map(|s| s.tier >= 4).unwrap_or(false) {
                shimmer(&ui_cx, 0.55);
            }
            self.cats.draw_overlay(&ui_cx, &self.cfg.cat.thanks_title, self.cfg.timing.reveal_seconds);
        }
        self.draw_hud(&ui_cx);
    }

    fn draw_shrine_world(&self, cx: &Cx, sprites: &Sprites, bg: &str) {
        if bg == "scene" {
            draw_backdrop(cx, sprites);
        } else {
            draw_platform(cx);
        }
        draw_particles(cx, &self.particles, true);
        let mut shrine_glow = 0.0;
        let mut cyl_on_altar = true;
        if let Some(s) = &self.current {
            let (w, a, r, o) = self.durations(s);
            let t = s.t;
            if s.tier >= 2 {
                let alpha = 1.0 - smooth((t - (w + a + r)) / o);
                draw_carpet(cx, t / 0.9, alpha, s.tier >= 4);
            }
            if t > w && t < w + a + r {
                shrine_glow = smooth((t - w - 0.8) / 1.2) * if s.action == Action::Pray { 1.0 } else { 0.5 };
            }
            if s.action == Action::Siamsi && t > w + 0.25 && t < w + a + r + 0.3 {
                cyl_on_altar = false;
            }
        }
        draw_shrine(cx, sprites, shrine_glow, cyl_on_altar);
        self.draw_current_and_line(cx, sprites);
        draw_foreground_flowers(cx);
    }

    fn draw_krathong_world(&self, cx: &Cx, sprites: &Sprites, bg: &str) {
        let scene = bg == "scene";
        if scene {
            kr::draw_sky(cx, sprites);
        }
        draw_particles(cx, &self.particles, true);
        kr::draw_river(cx, scene, scene);
        let vh = cx.vh;
        let wt = kr::water_top(vh);
        let mut fl: Vec<&Floating> = self.floating.iter().collect();
        fl.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
        for f in fl {
            let sc = lerp(0.6, 1.0, ((f.y - wt) / (vh - wt).max(1.0)).clamp(0.0, 1.0));
            let alpha = f.fade.map(|x| 1.0 - x / 2.0).unwrap_or(1.0).clamp(0.0, 1.0);
            let bob = (cx.time * 1.6 + f.seed).sin() * 2.5;
            kr::draw_krathong(cx, f.x, f.y + bob, sc, f.tier, f.seed, Some(&f.name), alpha);
        }
        kr::draw_bank(cx);
        self.draw_current_and_line(cx, sprites);
        kr::draw_foreground(cx);
    }

    /// the active character + the waiting line; a character walking away passes behind the line
    fn draw_current_and_line(&self, cx: &Cx, sprites: &Sprites) {
        let leaving = self.current.as_ref().map(|s| {
            let (w, a, r, _) = self.durations(s);
            s.t > w + a + r
        });
        if leaving == Some(true) {
            self.draw_actor(cx, sprites, self.current.as_ref().unwrap());
        }
        self.draw_line(cx, sprites);
        if leaving == Some(false) {
            self.draw_actor(cx, sprites, self.current.as_ref().unwrap());
        }
    }

    fn draw_actor(&self, cx: &Cx, sprites: &Sprites, s: &Show) {
        let (w, a_d, r, o) = self.durations(s);
        let (x, pose) = self.animate(s);
        let k = Self::char_scale(s.theme);
        if s.tier >= 4 {
            for (i, off) in [130.0f32, 230.0].iter().enumerate() {
                let mut p2 = pose.clone();
                p2.hold_cyl = None;
                p2.hands = 0.0;
                p2.carry = 0.0;
                p2.reach = None;
                p2.bow = 0.0;
                p2.kneel = 0.0;
                p2.phase += i as f32 * 1.3;
                p2.blink = blink(self.scene_time, s.look.seed + 3.0 + i as f32);
                p2.expr = if pose.expr == 1 { 1 } else { 0 };
                let ax = x - off * pose.facing;
                draw_attendant(cx, sprites, ax, &p2, i);
            }
        }
        let look = Look { tier: s.tier, shirt: s.look.shirt, pants: s.look.pants, scale: k, hair: s.look.hair, style: s.look.style };
        let rig = draw_character(cx, sprites, x, &pose, &look);
        // krathong being carried / placed / pushed
        if s.theme == Theme::Krathong && !s.released {
            let a = s.t - w;
            let held = s.t < w || a < 1.1;
            let kp = if held {
                let hd = rig.arms[1].1;
                (hd.0 + 6.0 * pose.facing, hd.1 + 4.0)
            } else {
                self.kr_pos(s, a)
            };
            kr::draw_krathong(cx, kp.0, kp.1, if held { 0.62 } else { 0.72 }, s.tier, s.look.seed, None, 1.0);
        }
        // fortune stick in flight / landed, with its number
        if let Some(st) = &s.stick {
            let landed = st.landed.map(|l| s.t - w - l);
            let end_fade = 1.0 - smooth((s.t - (w + a_d + r)) / 0.5);
            let glow = landed.map(|lt| 0.6 * smooth(lt / 0.3)).unwrap_or(0.0) * end_fade;
            if end_fade > 0.0 {
                draw_stick(cx, st.x, st.y, st.ang, glow);
            }
            if let Some(lt) = landed {
                let pop = ease_out_back(lt / 0.4);
                if end_fade > 0.0 && pop > 0.05 {
                    cx.text_o(st.x, st.y - 34.0 - 10.0 * pop, Align2::CENTER_CENTER, &s.stick_no.to_string(), 44.0 * pop, with_a(RED, end_fade), with_a(Color32::WHITE, end_fade), true);
                }
            }
        }
        // name banner (bounce-in, follows the head)
        let head_top = rig.head.1 - HEAD_R * k * 1.05 - if s.tier >= 3 { 70.0 * k } else { 8.0 };
        let total = w + a_d + r + o;
        let alpha = 1.0 - smooth((s.t - (total - 0.6)) / 0.6);
        draw_name_banner(cx, x, head_top, &s.user, s.tier, if s.tier >= 2 { &s.tier_name } else { "" }, if s.start_x > -149.0 { 1.0 } else { s.t / 0.9 }, alpha);
        tag_pointer(cx, x, head_top, s.look.tag, alpha);
        self.banner_at.set((x, head_top));
    }

    fn pray_geo(&self, s: &Show) -> PrayGeo {
        let rows = s.numbers.len();
        let tier = s.tier;
        let cxx = match s.theme {
            Theme::Krathong => 650.0,
            Theme::Cat => 540.0,
            Theme::Shrine => 430.0,
        };
        let top = 30.0;
        let row_h = 118.0;
        let hgt = 110.0 + rows as f32 * row_h;
        let wdt = 600.0 + tier as f32 * 20.0;
        let size = 82.0 + tier as f32 * 5.0;
        let mut row_y = vec![];
        let mut digit_x = vec![];
        for (i, (_, num)) in s.numbers.iter().enumerate() {
            row_y.push(top + 110.0 + i as f32 * row_h + 52.0);
            let n = num.chars().count();
            let dx = size * 0.78;
            let c = cxx + 95.0;
            digit_x.push((0..n).map(|j| c + (j as f32 - (n as f32 - 1.0) / 2.0) * dx).collect());
        }
        PrayGeo { cxx, top, wdt, hgt, row_y, digit_x, size }
    }

    fn draw_reveal(&self, cx: &Cx, s: &Show) {
        let (w, a, r, _o) = self.durations(s);
        let rt = s.t - w - a;
        if rt < 0.0 || rt > r {
            return;
        }
        match (s.action, s.theme) {
            // cat mode never shows lucky numbers: its thank-you card is drawn by cat::Stage
            (_, Theme::Cat) => {}
            (Action::Pray, _) => self.draw_number_card(cx, s, rt, r),
            (Action::Siamsi, Theme::Shrine) => self.draw_scroll(cx, s, rt, r),
            (Action::Siamsi, Theme::Krathong) => self.draw_blessing(cx, s, rt, r),
        }
    }

    fn draw_number_card(&self, cx: &Cx, s: &Show, rt: f32, r: f32) {
        let geo = self.pray_geo(s);
        let tier = s.tier;
        let fo = smooth((rt - (r - 0.5)) / 0.5);
        let alpha = smooth(rt / 0.25) * (1.0 - fo);
        let yoff = -(1.0 - ease_out_back(rt / 0.6)) * (geo.hgt + 60.0) - fo * 40.0;
        let (x0, x1) = (geo.cxx - geo.wdt / 2.0, geo.cxx + geo.wdt / 2.0);
        let top = geo.top + yoff;
        let border = if tier >= 2 { GOLD } else { rgba(255, 220, 150, 1.0) };
        if tier >= 3 {
            cx.glow(geo.cxx, top + geo.hgt / 2.0, geo.wdt * 0.8, GOLD, 0.4 * alpha);
        }
        cx.rect(x0 + 8.0, top + 12.0, x1 + 8.0, top + geo.hgt + 12.0, 26.0, rgba(0, 0, 0, 0.3 * alpha));
        let (c_top, c_bot) = match s.theme {
            Theme::Krathong => (rgba(36, 48, 120, 0.96 * alpha), rgba(14, 18, 64, 0.96 * alpha)),
            Theme::Cat => (rgba(255, 236, 214, 0.96 * alpha), rgba(255, 210, 180, 0.96 * alpha)),
            Theme::Shrine => (rgba(170, 26, 46, 0.96 * alpha), rgba(96, 8, 24, 0.96 * alpha)),
        };
        cx.vgradient(x0, top, x1, top + geo.hgt, c_top, c_bot);
        cx.rect_line(x0, top, x1, top + geo.hgt, 18.0, 5.0 + tier as f32, with_a(border, alpha));
        cx.rect_line(x0 + 12.0, top + 12.0, x1 - 12.0, top + geo.hgt - 12.0, 12.0, 2.0, with_a(border, 0.55 * alpha));
        for (ox, oy) in [(x0 + 12.0, top + 12.0), (x1 - 12.0, top + 12.0), (x0 + 12.0, top + geo.hgt - 12.0), (x1 - 12.0, top + geo.hgt - 12.0)] {
            cx.poly(&[(ox, oy - 9.0), (ox + 9.0, oy), (ox, oy + 9.0), (ox - 9.0, oy)], with_a(border, alpha));
        }
        let title = format!("{} ของ {}", self.cfg.numbers.title, s.user);
        cx.text_o(geo.cxx, top + 55.0, Align2::CENTER_CENTER, &title, 38.0, with_a(GOLD, alpha), rgba(60, 0, 0, alpha), true);
        self.owner_tag(cx, s, x0, x1, top, top + geo.hgt, alpha);
        for (i, (label, num)) in s.numbers.iter().enumerate() {
            let y = geo.row_y[i] + yoff;
            cx.text(x0 + 46.0, y, Align2::LEFT_CENTER, label, 32.0, rgba(255, 235, 200, alpha), true);
            let start = row_start(i);
            let digits: Vec<char> = num.chars().collect();
            let bw = geo.size * 0.66;
            let bh = geo.size * 1.08;
            for (j, ch) in digits.iter().enumerate() {
                let x = geo.digit_x[i][j];
                cx.rect(x - bw / 2.0, y - bh / 2.0, x + bw / 2.0, y + bh / 2.0, 10.0, rgba(0, 0, 0, 0.35 * alpha * smooth((rt - start + 0.3) / 0.3)));
                if rt < start {
                    continue;
                }
                let stop = digit_stop(i, j);
                if rt < stop {
                    // slot-machine strip clipped to the slot; slows slightly before stopping
                    let v = (rt - start) * 16.0 + j as f32 * 3.3 + i as f32 * 1.7;
                    let d = (v.floor() as i64).rem_euclid(10) as u8;
                    let fr = v.fract();
                    let clip = cx.clip_rect_v(x - bw / 2.0, y - bh / 2.0, x + bw / 2.0, y + bh / 2.0);
                    let cp = cx.p.with_clip_rect(clip);
                    let c2 = cx.with_painter(&cp);
                    let col = rgba(255, 225, 140, 0.85 * alpha);
                    c2.text(x, y + fr * bh, Align2::CENTER_CENTER, &((b'0' + d) as char).to_string(), geo.size * 0.9, col, true);
                    c2.text(x, y + fr * bh - bh, Align2::CENTER_CENTER, &((b'0' + (d + 1) % 10) as char).to_string(), geo.size * 0.9, col, true);
                } else {
                    let st = rt - stop;
                    let sc = lerp(1.7, 1.0, ease_out_back(st / 0.35));
                    let flash = (1.0 - st / 0.4).max(0.0);
                    let pulse = 0.25 + 0.1 * (cx.time * 4.0 + j as f32).sin();
                    cx.glow_add(x, y, geo.size * 0.9, GOLD, (pulse + flash * 0.8) * alpha);
                    cx.text_o(x, y, Align2::CENTER_CENTER, &ch.to_string(), geo.size * sc, with_a(Color32::from_rgb(255, 228, 100), alpha), rgba(90, 20, 0, alpha), true);
                }
            }
        }
    }

    fn draw_scroll(&self, cx: &Cx, s: &Show, rt: f32, r: f32) {
        let tier = s.tier;
        let cxx = 430.0;
        let wdt = 640.0 + tier as f32 * 16.0;
        let (x0, x1) = (cxx - wdt / 2.0, cxx + wdt / 2.0);
        let full = 400.0;
        let fo = smooth((rt - (r - 0.5)) / 0.5);
        let alpha = smooth(rt / 0.2) * (1.0 - fo);
        let top = 30.0 - fo * 30.0;
        let h = full * ease_out_cubic(rt / 0.9) * (1.0 - 0.6 * fo);
        if tier >= 3 {
            cx.glow(cxx, top + h / 2.0, wdt * 0.8, GOLD, 0.4 * alpha * (h / full));
        }
        cx.rect(x0 + 8.0, top + 10.0, x1 + 8.0, top + h + 10.0, 16.0, rgba(0, 0, 0, 0.3 * alpha));
        cx.vgradient(x0, top, x1, top + h, rgba(255, 248, 225, 0.98 * alpha), rgba(245, 222, 170, 0.98 * alpha));
        cx.rect_line(x0, top, x1, top + h, 12.0, 4.0 + tier as f32, with_a(if tier >= 2 { GOLD_D } else { RED }, alpha));
        let clip = cx.clip_rect_v(x0, top, x1, top + h);
        let cp = cx.p.with_clip_rect(clip);
        let c2 = cx.with_painter(&cp);
        let pop = ease_out_back((rt - 0.45) / 0.4);
        let title = format!("{} {}", self.cfg.siamsi.title, s.stick_no);
        if pop > 0.01 {
            c2.text_o(cxx, top + 78.0, Align2::CENTER_CENTER, &title, 56.0 * pop, rgba(170, 10, 30, alpha), rgba(255, 255, 255, alpha), true);
        }
        c2.seg((x0 + 60.0, top + 128.0), (x1 - 60.0, top + 128.0), 3.0, with_a(GOLD_D, alpha));
        let ta = alpha * smooth((rt - 0.8) / 0.5);
        let rise = (1.0 - smooth((rt - 0.8) / 0.5)) * 16.0;
        c2.text_wrapped(cxx, top + 152.0 + rise, &s.text, 36.0, wdt - 100.0, rgba(70, 30, 10, ta), true);
        c2.text(cxx, top + full - 40.0, Align2::CENTER_CENTER, &format!("คำทำนายของ {}", s.user), 26.0, rgba(140, 60, 20, ta), false);
        let rod = |y: f32| {
            cx.rect(x0 - 18.0, y - 11.0, x1 + 18.0, y + 11.0, 10.0, rgba(140, 20, 30, alpha));
            cx.rect(x0 - 18.0, y - 11.0, x1 + 18.0, y - 4.0, 8.0, rgba(190, 50, 60, alpha));
            cx.circle(x0 - 22.0, y, 13.0, with_a(GOLD, alpha));
            cx.circle(x1 + 22.0, y, 13.0, with_a(GOLD, alpha));
        };
        rod(top);
        rod(top + h);
        self.owner_tag(cx, s, x0, x1, top, top + h + 11.0, alpha);
    }

    fn draw_blessing(&self, cx: &Cx, s: &Show, rt: f32, r: f32) {
        let tier = s.tier;
        let cxx = 650.0;
        let wdt = 620.0 + tier as f32 * 16.0;
        let hgt = 310.0;
        let fo = smooth((rt - (r - 0.5)) / 0.5);
        let alpha = smooth(rt / 0.25) * (1.0 - fo);
        let sc = ease_out_elastic(rt / 1.0).max(0.01) * (1.0 - 0.1 * fo);
        let top = 30.0 + hgt * (1.0 - sc) / 2.0 - fo * 30.0;
        let (w2, h2) = (wdt * sc, hgt * sc);
        let (x0, x1) = (cxx - w2 / 2.0, cxx + w2 / 2.0);
        cx.glow(cxx, top + h2 / 2.0, w2 * 0.75, rgba(255, 210, 140, 1.0), 0.35 * alpha);
        cx.rect(x0 + 8.0, top + 10.0, x1 + 8.0, top + h2 + 10.0, 24.0, rgba(0, 0, 0, 0.3 * alpha));
        cx.vgradient(x0, top, x1, top + h2, rgba(36, 46, 124, 0.96 * alpha), rgba(16, 18, 66, 0.96 * alpha));
        cx.rect_line(x0, top, x1, top + h2, 24.0, 4.0 + tier as f32, with_a(GOLD, alpha));
        if sc < 0.6 {
            return;
        }
        let ca = alpha * smooth((rt - 0.35) / 0.4);
        kr::draw_krathong(cx, x0 + 64.0, top + 74.0, 0.5, tier, s.look.seed, None, ca);
        cx.text_o(cxx + 20.0, top + 62.0, Align2::CENTER_CENTER, &format!("{} {}", self.cfg.krathong.wish_title, s.user), 36.0, with_a(GOLD, ca), rgba(10, 10, 40, ca), true);
        cx.seg((x0 + 70.0, top + 112.0), (x1 - 70.0, top + 112.0), 2.0, with_a(GOLD, ca * 0.8));
        self.owner_tag(cx, s, x0, x1, top, top + h2, ca);
        let ta = alpha * smooth((rt - 0.7) / 0.5);
        cx.text_wrapped(cxx, top + 140.0 + (1.0 - smooth((rt - 0.7) / 0.5)) * 14.0, &s.text, 38.0, w2 - 110.0, rgba(255, 245, 220, ta), true);
        for i in 0..10 {
            let a = i as f32 / 10.0 * std::f32::consts::TAU + cx.time * 0.3;
            let tw = (cx.time * 3.0 + i as f32 * 1.7).sin().abs();
            sparkle(cx, cxx + a.cos() * (w2 / 2.0 + 10.0), top + h2 / 2.0 + a.sin() * (h2 / 2.0 + 10.0), 9.0 * tw, rgba(255, 240, 200, ca));
        }
    }

    /// whose result is this: a name tab in the sender's colour on the card + a dotted pointer down to their name label
    fn owner_tag(&self, cx: &Cx, s: &Show, x0: f32, x1: f32, top: f32, bottom: f32, alpha: f32) {
        if alpha <= 0.02 {
            return;
        }
        let col = s.look.tag;
        let (bx, by) = self.banner_at.get();
        // pointer from the card's bottom edge to just above the character's name label
        let sx = bx.clamp(x0 + 40.0, x1 - 40.0);
        let (ax, ay) = (sx, bottom + 6.0);
        let (ex, ey) = (bx, by - 62.0);
        if ey > ay + 20.0 {
            let n = (((ex - ax).hypot(ey - ay)) / 16.0) as usize;
            for i in 0..n {
                if i % 2 == 0 {
                    let (u0, u1) = (i as f32 / n as f32, (i as f32 + 1.0) / n as f32);
                    cx.seg((lerp(ax, ex, u0), lerp(ay, ey, u0)), (lerp(ax, ex, u1), lerp(ay, ey, u1)), 4.0, rgba(255, 255, 255, 0.85 * alpha));
                }
            }
            cx.circle(ex, ey, 7.0, rgba(255, 255, 255, 0.9 * alpha));
            cx.circle(ex, ey, 4.5, with_a(col, alpha));
        }
        // name tab hanging off the card's top-left corner
        let name = truncate(&s.user, 16);
        let size = 24.0;
        let m = cx.measure(&name, size, true);
        let (tx0, ty0) = (x0 + 26.0, top - 20.0);
        let (tx1, ty1) = (tx0 + m.x + 54.0, top + 16.0);
        cx.rect(tx0 + 3.0, ty0 + 4.0, tx1 + 3.0, ty1 + 4.0, 18.0, rgba(0, 0, 0, 0.35 * alpha));
        cx.rect(tx0, ty0, tx1, ty1, 18.0, with_a(col, alpha));
        cx.rect_line(tx0, ty0, tx1, ty1, 18.0, 2.5, rgba(255, 255, 255, 0.95 * alpha));
        cx.circle(tx0 + 20.0, (ty0 + ty1) / 2.0, 7.0, rgba(255, 255, 255, 0.95 * alpha));
        cx.text_o(tx0 + 34.0, (ty0 + ty1) / 2.0, Align2::LEFT_CENTER, &name, size, rgba(255, 255, 255, alpha), rgba(0, 0, 0, 0.85 * alpha), true);
    }

    fn draw_hud(&self, cx: &Cx) {
        let waiting: Vec<&str> = if self.theme() == Theme::Cat {
            self.cats.waiting_names()
        } else {
            self.queue.iter().map(|s| s.user.as_str()).collect()
        };
        if !waiting.is_empty() {
            let names = waiting.iter().take(3).copied().collect::<Vec<_>>();
            let more = waiting.len().saturating_sub(3);
            let txt = if more > 0 { format!("ถัดไป: {}  +{}", names.join(", "), more) } else { format!("ถัดไป: {}", names.join(", ")) };
            let m = cx.measure(&txt, 24.0, true);
            cx.rect(16.0, cx.vh - 60.0, 44.0 + m.x, cx.vh - 16.0, 22.0, rgba(0, 0, 0, 0.55));
            cx.text(30.0, cx.vh - 38.0, Align2::LEFT_CENTER, &txt, 24.0, Color32::WHITE, true);
        }
        let idle = self.current.is_none() && self.queue.is_empty() && !self.cats.busy();
        if self.cfg.window.show_status && !self.status.is_empty() && idle {
            cx.text_o(VW - 16.0, 24.0, Align2::RIGHT_CENTER, &self.status, 20.0, Color32::WHITE, rgba(0, 0, 0, 0.8), false);
        }
        if !self.running {
            cx.text_o(VW - 16.0, 56.0, Align2::RIGHT_CENTER, "หยุดรับของขวัญชั่วคราว", 22.0, rgba(255, 120, 120, 1.0), rgba(0, 0, 0, 0.8), true);
        }
        let hint = match self.theme() {
            Theme::Krathong => &self.cfg.krathong.hint_text,
            Theme::Cat => &self.cfg.cat.hint_text,
            Theme::Shrine => &self.cfg.window.hint_text,
        };
        if self.cfg.window.show_hint && idle && !hint.is_empty() {
            let y = if matches!(self.theme(), Theme::Krathong | Theme::Cat) { 36.0 } else { cx.vh - 40.0 };
            cx.text_o(VW / 2.0, y, Align2::CENTER_CENTER, hint, 28.0, GOLD, rgba(60, 0, 10, 0.95), true);
        }
        if self.flash_msg.1 > 0.0 {
            cx.text_o(VW / 2.0, 30.0, Align2::CENTER_CENTER, &self.flash_msg.0, 24.0, Color32::WHITE, rgba(0, 0, 0, 0.9), true);
        }
    }

    fn publish_state(&mut self) {
        self.frame += 1;
        if self.frame % 6 != 0 {
            return;
        }
        let cur = if self.theme() == Theme::Cat {
            let rv = self.cfg.timing.reveal_seconds;
            // cat mode: the result is always a thank-you card, never lucky numbers
            self.cats.shows().next().map(|s| serde_json::json!({"action": format!("{:?}", s.action), "theme": "cat", "user": s.user, "tier": s.tier + 1, "t": s.t, "phase": s.phase(rv), "card": "thanks"}))
        } else {
            self.current.as_ref().map(|s| {
                let card = match (s.action, s.theme) {
                    (_, Theme::Cat) => "thanks",
                    (Action::Pray, _) => "numbers",
                    (Action::Siamsi, Theme::Shrine) => "fortune",
                    (Action::Siamsi, Theme::Krathong) => "blessing",
                };
                serde_json::json!({"action": format!("{:?}", s.action), "theme": s.theme.key(), "user": s.user, "tier": s.tier + 1, "t": s.t, "card": card})
            })
        };
        let queued: Vec<serde_json::Value> = if self.theme() == Theme::Cat {
            self.cats.shows().skip(1).map(|s| serde_json::json!({"action": format!("{:?}", s.action), "user": s.user, "tier": s.tier + 1})).collect()
        } else {
            self.queue.iter().map(|s| serde_json::json!({"action": format!("{:?}", s.action), "user": s.user, "tier": s.tier + 1})).collect()
        };
        let (w, h) = self.cfg.window_size();
        let v = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "theme": self.theme().key(),
            "running": self.running,
            "idle": !self.active(),
            "idle_mode": self.cfg.window.idle,
            "scene_alpha": self.scene_alpha,
            "scene_time": self.scene_time,
            "background": self.cfg.window.background,
            "layout": self.cfg.window.layout,
            "window_size": [w, h],
            "always_on_top": self.cfg.window.always_on_top,
            "click_through": self.cfg.window.click_through,
            "test_tier": self.test_tier + 1,
            "current": cur,
            "queue": queued,
            "queue_len": if self.theme() == Theme::Cat { self.cats.len().saturating_sub(1) } else { self.queue.len() },
            "line": if self.theme() == Theme::Cat {
                self.cats.shows().skip(1).map(|s| serde_json::json!({"user": s.user, "x": s.x.round()})).collect::<Vec<_>>()
            } else {
                self.queue.iter().filter_map(|s| s.qx.map(|x| serde_json::json!({"user": s.user, "x": x.round()}))).collect::<Vec<_>>()
            },
            "floating": self.floating.len(),
            "particles": self.particles.len(),
            "shows_started": self.shows_started,
            "shows_finished": self.shows_finished,
            "gifts_received": self.gifts_received,
            "settings_open": self.settings_open,
            "frame": self.frame,
            "time": self.time,
            "config_path": self.cfg_path.display().to_string(),
            "update": self.updater.snapshot_json(),
        });
        if let Ok(mut s) = self.state.lock() {
            *s = v.to_string();
        }
    }
}

struct PrayGeo {
    cxx: f32,
    top: f32,
    wdt: f32,
    hgt: f32,
    row_y: Vec<f32>,
    digit_x: Vec<Vec<f32>>,
    size: f32,
}

/// stick pops out of the cylinder at this act-time
const POP: f32 = 3.5;
fn row_start(i: usize) -> f32 {
    0.45 + i as f32 * 0.25
}
fn digit_stop(i: usize, j: usize) -> f32 {
    row_start(i) + 0.75 + j as f32 * 0.32 + i as f32 * 0.35
}
fn kr_wish_window(a: Action) -> (f32, f32) {
    match a {
        Action::Pray => (1.4, 2.9),
        Action::Siamsi => (1.4, 3.5),
    }
}
fn kr_push_end(a: Action) -> f32 {
    kr_wish_window(a).1 + 0.9
}

fn blink(t: f32, seed: f32) -> f32 {
    let period = 3.1 + (seed * 7.3).fract() * 1.4;
    let ph = (t + seed * 5.0).rem_euclid(period);
    if ph < 0.13 { 1.0 - ((ph / 0.13) * 2.0 - 1.0).abs() } else { 0.0 }
}

/// Per-viewer look from a hash of the nickname (each viewer looks different).
pub fn appearance_for(name: &str) -> Appearance {
    let mut h: u64 = 1469598103934665603;
    for b in name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    let hue = (h % 360) as f32 / 360.0;
    let hairs = [Color32::from_rgb(52, 34, 28), Color32::from_rgb(96, 58, 38), Color32::from_rgb(30, 28, 40), Color32::from_rgb(140, 84, 48), Color32::from_rgb(70, 44, 74)];
    Appearance {
        shirt: hsv(hue, 0.35, 0.98),
        pants: hsv((hue + 0.5).fract(), 0.45, 0.78),
        hair: hairs[((h >> 12) % hairs.len() as u64) as usize],
        style: ((h >> 20) % 4) as u8,
        seed: ((h >> 28) % 1000) as f32 / 100.0,
        tag: hsv(hue, 0.85, 0.78),
    }
}


impl ShrineApp {
    // ------------------------------------------------------------------ input
    fn handle_keys(&mut self, ctx: &egui::Context) {
        let (keys, shift, cmd) = ctx.input(|i| {
            let ks: Vec<Key> = i.events.iter().filter_map(|e| if let egui::Event::Key { key, pressed: true, repeat: false, .. } = e { Some(*key) } else { None }).collect();
            (ks, i.modifiers.shift, i.modifiers.command)
        });
        for k in keys {
            let tier_keys_p = [Key::Q, Key::W, Key::E, Key::R, Key::T];
            let tier_keys_s = [Key::A, Key::S, Key::D, Key::F, Key::G];
            let nums = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5];
            if cmd && k == Key::Q {
                ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
            } else if shift && nums.contains(&k) {
                self.test_tier = nums.iter().position(|x| *x == k).unwrap();
                self.flash(format!("ระดับทดสอบ: {} ({})", self.test_tier + 1, self.cfg.tiers.get(self.test_tier).map(|t| t.name.as_str()).unwrap_or("")));
            } else if k == Key::Num1 {
                { let n = test_name(&mut self.rng); self.enqueue(Action::Pray, &n, self.tier_min(self.test_tier), self.test_tier); }
            } else if k == Key::Num2 {
                { let n = test_name(&mut self.rng); self.enqueue(Action::Siamsi, &n, self.tier_min(self.test_tier), self.test_tier); }
            } else if let Some(i) = tier_keys_p.iter().position(|x| *x == k) {
                { let n = test_name(&mut self.rng); self.enqueue(Action::Pray, &n, self.tier_min(i), i); }
            } else if let Some(i) = tier_keys_s.iter().position(|x| *x == k) {
                { let n = test_name(&mut self.rng); self.enqueue(Action::Siamsi, &n, self.tier_min(i), i); }
            } else if k == Key::F1 || k == Key::Comma {
                self.settings_open = !self.settings_open;
            } else if k == Key::F8 {
                self.cfg.window.click_through = !self.cfg.window.click_through;
                self.flash(if self.cfg.window.click_through { "คลิกทะลุ: เปิด".into() } else { "คลิกทะลุ: ปิด".into() });
            } else if k == Key::F9 {
                let next = match self.cfg.window.background.as_str() {
                    "transparent" => "green",
                    "green" => "scene",
                    _ => "transparent",
                };
                self.cfg.window.background = next.into();
            } else if k == Key::F2 {
                self.sync_edit_buffers();
                let next = self.theme().next();
                self.cfg.theme.mode = next.key().into();
                self.load_gift_rows();
                self.flash(format!("โหมด: {}", next.title()));
            } else if k == Key::Delete {
                self.queue.clear();
                self.cats.clear();
            }
        }
    }

    fn flash(&mut self, msg: String) {
        self.flash_msg = (msg, 2.0);
    }

    fn apply_window(&mut self, ctx: &egui::Context) {
        let now = Applied { size: self.cfg.window_size(), aot: self.cfg.window.always_on_top, click: self.cfg.window.click_through };
        let prev = self.applied;
        if prev.map(|p| p.size) != Some(now.size) && prev.is_some() {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::InnerSize(Vec2::new(now.size.0, now.size.1)));
        }
        if prev.map(|p| p.aot) != Some(now.aot) {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::WindowLevel(if now.aot { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal }));
        }
        if prev.map(|p| p.click) != Some(now.click) {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::MousePassthrough(now.click));
        }
        self.applied = Some(now);
    }

    fn save_config(&mut self, ctx: &egui::Context) {
        if let Some(r) = ctx.input(|i| i.viewport().outer_rect) {
            self.cfg.window.x = Some(r.min.x.round());
            self.cfg.window.y = Some(r.min.y.round());
        }
        self.sync_edit_buffers();
        match self.cfg.save(&self.cfg_path) {
            Ok(()) => self.flash(format!("บันทึกแล้ว: {}", self.cfg_path.display())),
            Err(e) => self.flash(format!("บันทึกไม่สำเร็จ: {e}")),
        }
    }

    fn sync_edit_buffers(&mut self) {
        let map: std::collections::BTreeMap<String, String> = self.gift_rows.iter().filter(|(k, _)| !k.trim().is_empty()).map(|(k, v)| (k.trim().to_string(), v.clone())).collect();
        match self.gift_rows_theme {
            Theme::Shrine => self.cfg.gifts = map,
            Theme::Krathong => self.cfg.krathong.gifts = map,
            Theme::Cat => self.cfg.cat.gifts = map,
        }
        let f: Vec<String> = self.fortunes_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        if !f.is_empty() {
            self.cfg.siamsi.fortunes = f;
        }
        let b: Vec<String> = self.blessings_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        if !b.is_empty() {
            self.cfg.krathong.blessings = b;
        }
        let thx: Vec<String> = self.thanks_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        if !thx.is_empty() {
            self.cfg.cat.thanks = thx;
        }
    }

    fn reconnect(&mut self) {
        restart_bridge(&self.bridge, &self.cfg.tiktok.username, &self.cfg.tiktok.sign_api_key, &self.cfg.tiktok.node_command, self.running && self.cfg.tiktok.enabled);
    }

    // ------------------------------------------------------------------ settings window
    fn settings_ui(&mut self, ctx: &egui::Context) {
        let mut open = self.settings_open;
        let vb = ViewportBuilder::default().with_title("ตั้งค่า - TikTok Shrine Live").with_inner_size([600.0, 720.0]).with_min_inner_size([480.0, 400.0]);
        ctx.show_viewport_immediate(ViewportId::from_hash_of("settings"), vb, |ui, _class| {
            let ctx = &ui.ctx().clone();
            if ctx.input(|i| i.viewport().close_requested()) {
                open = false;
            }
            self.handle_keys(ctx);
            egui::CentralPanel::default().show(ui, |ui| {
                self.settings_contents(ui, ctx);
            });
            if let Some(shot) = &mut self.shot {
                if shot.settings_tab.is_some() && self.time >= shot.at && !shot.requested {
                    shot.requested = true;
                    let out = shot.out.clone();
                    let done = shot.done.clone();
                    ctx.request_screenshot(move |img| {
                        save_png(&img, &out, false);
                        done.store(true, Ordering::SeqCst);
                    });
                }
            }
        });
        self.settings_open = open;
    }

    fn settings_contents(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let tabs = ["เริ่ม/หยุด", "TikTok", "หน้าจอ", "ของขวัญ", "ระดับ", "เลข/คำทำนาย", "ทดสอบ", "อัปเดต"];
        ui.horizontal_wrapped(|ui| {
            for (i, t) in tabs.iter().enumerate() {
                if ui.selectable_label(self.settings_tab == i, egui::RichText::new(*t).size(17.0)).clicked() {
                    self.settings_tab = i;
                }
            }
        });
        self.update_banner_ui(ui);
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - 52.0).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            match self.settings_tab {
                0 => self.tab_main(ui, ctx),
                1 => self.tab_tiktok(ui),
                2 => self.tab_window(ui),
                3 => self.tab_gifts(ui),
                4 => self.tab_tiers(ui),
                5 => self.tab_numbers(ui),
                6 => self.tab_test(ui),
                _ => self.tab_update(ui),
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("บันทึกลง config.toml").size(16.0)).clicked() {
                self.save_config(ctx);
            }
            if ui.button(egui::RichText::new("โหลดค่าใหม่").size(16.0)).clicked() {
                self.reload_config();
            }
            if self.flash_msg.1 > 0.0 {
                ui.label(egui::RichText::new(&self.flash_msg.0).color(Color32::from_rgb(120, 220, 120)));
            }
        });
    }

    fn tab_main(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("โหมด:").size(18.0).strong());
            let cur = self.theme();
            for (th, label) in [(Theme::Shrine, "ไหว้พระ / เสี่ยงเซียมซี"), (Theme::Krathong, "ลอยกระทง"), (Theme::Cat, "ให้อาหารแมว")] {
                let sel = cur == th;
                let b = egui::Button::new(egui::RichText::new(label).size(16.0).strong().color(if sel { Color32::WHITE } else { Color32::LIGHT_GRAY })).fill(if sel { Color32::from_rgb(170, 110, 20) } else { Color32::from_gray(50) }).min_size(Vec2::new(170.0, 40.0)).corner_radius(10);
                if ui.add(b).clicked() && !sel {
                    self.sync_edit_buffers();
                    self.cfg.theme.mode = th.key().into();
                    self.load_gift_rows();
                }
            }
        });
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            let (txt, col) = if self.running { ("หยุดรับของขวัญ", Color32::from_rgb(200, 40, 50)) } else { ("เริ่มรับของขวัญ", Color32::from_rgb(30, 150, 70)) };
            let btn = egui::Button::new(egui::RichText::new(txt).size(30.0).strong().color(Color32::WHITE)).fill(col).min_size(Vec2::new(380.0, 84.0)).corner_radius(14);
            if ui.add(btn).clicked() {
                self.running = !self.running;
                self.reconnect();
            }
            ui.add_space(4.0);
            let st = if self.running { "กำลังทำงาน - รับของขวัญอยู่" } else { "หยุดชั่วคราว - ไม่รับของขวัญ" };
            ui.label(egui::RichText::new(st).size(18.0).strong());
            ui.label(egui::RichText::new(format!("สถานะ TikTok: {}", if self.status.is_empty() { "-" } else { &self.status })).size(16.0));
            let (qlen, playing) = if self.theme() == Theme::Cat {
                (self.cats.len().saturating_sub(if self.cats.front_user().is_some() { 1 } else { 0 }), self.cats.front_user().unwrap_or("-"))
            } else {
                (self.queue.len(), self.current.as_ref().map(|s| s.user.as_str()).unwrap_or("-"))
            };
            ui.label(egui::RichText::new(format!("คิว: {qlen} คน   |   กำลังเล่น: {playing}")).size(16.0));
        });
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("ปุ่มลัด (คลิกที่หน้าต่างเอฟเฟกต์ก่อน)").strong());
            ui.label("1 = การกระทำที่ 1 (ไหว้พระ / ลอยกระทง / ให้อาหารแมว)   2 = การกระทำที่ 2 (เซียมซี / อธิษฐาน / แมวขอบคุณ)   Shift+1..5 = เลือกระดับทดสอบ");
            ui.label("Q W E R T = การกระทำที่ 1 ระดับ 1-5   |   A S D F G = การกระทำที่ 2 ระดับ 1-5   F2 = สลับโหมด");
            ui.label("F1 หรือ คลิกขวา = เปิด/ปิดหน้าตั้งค่า   F8 = คลิกทะลุ   F9 = สลับพื้นหลัง");
            ui.label("Delete = ล้างคิว   Ctrl+Q (Mac: Cmd+Q) = ปิดโปรแกรม   ลากหน้าต่างเพื่อย้ายตำแหน่ง");
        });
        ui.add_space(10.0);
        ui.vertical_centered(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("ปิดโปรแกรม").size(18.0)).min_size(Vec2::new(200.0, 40.0))).clicked() {
                ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
            }
        });
    }

    fn tab_tiktok(&mut self, ui: &mut egui::Ui) {
        ui.heading("เชื่อมต่อ TikTok LIVE");
        egui::Grid::new("tt").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
            ui.label("ชื่อผู้ใช้ TikTok (@)");
            ui.add(egui::TextEdit::singleline(&mut self.cfg.tiktok.username).hint_text("เช่น mychannel").desired_width(260.0));
            ui.end_row();
            ui.label("เชื่อมต่ออัตโนมัติ");
            ui.checkbox(&mut self.cfg.tiktok.enabled, "เปิด");
            ui.end_row();
            ui.label("EulerStream API key");
            ui.add(egui::TextEdit::singleline(&mut self.cfg.tiktok.sign_api_key).password(true).hint_text("ไม่บังคับ").desired_width(260.0));
            ui.end_row();
            ui.label("คำสั่ง Node.js");
            ui.add(egui::TextEdit::singleline(&mut self.cfg.tiktok.node_command).desired_width(260.0));
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("เชื่อมต่อ / เชื่อมต่อใหม่").size(17.0)).min_size(Vec2::new(240.0, 38.0))).clicked() {
                self.reconnect();
            }
        });
        ui.label(egui::RichText::new(format!("สถานะ: {}", if self.status.is_empty() { "-" } else { &self.status })).size(16.0).strong());
        ui.separator();
        ui.label("ต้องเปิดไลฟ์อยู่ก่อนถึงจะเชื่อมต่อได้ และต้องติดตั้ง Node.js (https://nodejs.org) ในเครื่อง");
        ui.label(format!("หรือใช้ TikFinity / Streamer.bot ส่ง Webhook มาที่ http://{}/event", self.cfg.http.bind));
    }

    fn tab_window(&mut self, ui: &mut egui::Ui) {
        ui.heading("หน้าต่างเอฟเฟกต์");
        egui::Grid::new("win").num_columns(2).spacing([12.0, 10.0]).show(ui, |ui| {
            ui.label("เลย์เอาต์");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.cfg.window.layout, "top".to_string(), "ส่วนบนของจอ");
                ui.radio_value(&mut self.cfg.window.layout, "full".to_string(), "เต็มจอ 9:16");
            });
            ui.end_row();
            ui.label("ความสูง (% ของจอ)");
            ui.add_enabled(self.cfg.window.layout == "top", egui::Slider::new(&mut self.cfg.window.top_percent, 20.0..=100.0).suffix(" %").step_by(5.0));
            ui.end_row();
            ui.label("ความกว้าง (px)");
            ui.add(egui::Slider::new(&mut self.cfg.window.width, 270.0..=1440.0).step_by(10.0));
            ui.end_row();
            let (w, h) = self.cfg.window_size();
            ui.label("ขนาดจริง");
            ui.label(format!("{w} x {h} px"));
            ui.end_row();
            ui.label("ตำแหน่ง X / Y");
            let pe = self.pos_edit.get_or_insert((self.cfg.window.x.unwrap_or(100.0), self.cfg.window.y.unwrap_or(60.0)));
            let mut moved = false;
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut pe.0).speed(2.0));
                ui.add(egui::DragValue::new(&mut pe.1).speed(2.0));
                moved = ui.button("ย้าย").clicked();
            });
            if moved {
                let p = *pe;
                self.cfg.window.x = Some(p.0);
                self.cfg.window.y = Some(p.1);
                ui.ctx().send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::OuterPosition(Pos2::new(p.0, p.1)));
            }
            ui.end_row();
            ui.label("พื้นหลัง");
            ui.vertical(|ui| {
                ui.radio_value(&mut self.cfg.window.background, "transparent".to_string(), "โปร่งใส (ลอยทับจอ)");
                ui.radio_value(&mut self.cfg.window.background, "green".to_string(), "สีเขียว (Chroma Key)");
                ui.radio_value(&mut self.cfg.window.background, "scene".to_string(), "ฉากเต็มพื้นหลัง (วัด / แม่น้ำยามค่ำ / มุมแมว)");
            });
            ui.end_row();
            ui.label("ลอยอยู่บนสุดเสมอ");
            ui.checkbox(&mut self.cfg.window.always_on_top, "");
            ui.end_row();
            ui.label("คลิกทะลุหน้าต่าง");
            ui.checkbox(&mut self.cfg.window.click_through, "(F8 สลับ)");
            ui.end_row();
            ui.label("ตอนว่าง (ไม่มีของขวัญ)");
            ui.vertical(|ui| {
                ui.radio_value(&mut self.cfg.window.idle, "still".to_string(), "ภาพนิ่ง (ค่าเริ่มต้น)");
                ui.radio_value(&mut self.cfg.window.idle, "ambient".to_string(), "ขยับเบา ๆ (เปลวเทียน ควันธูป)");
                ui.radio_value(&mut self.cfg.window.idle, "hidden".to_string(), "ซ่อน (โปร่งใสทั้งหมด)");
            });
            ui.end_row();
            ui.label("แสดงคำแนะนำ");
            ui.checkbox(&mut self.cfg.window.show_hint, "");
            ui.end_row();
            ui.label("ข้อความคำแนะนำ");
            match self.theme() {
                Theme::Krathong => { ui.add(egui::TextEdit::singleline(&mut self.cfg.krathong.hint_text).desired_width(330.0)); }
                Theme::Cat => { ui.add(egui::TextEdit::singleline(&mut self.cfg.cat.hint_text).desired_width(330.0)); }
                Theme::Shrine => { ui.add(egui::TextEdit::singleline(&mut self.cfg.window.hint_text).desired_width(330.0)); }
            }
            ui.end_row();
            ui.label("แสดงสถานะการเชื่อมต่อ");
            ui.checkbox(&mut self.cfg.window.show_status, "");
            ui.end_row();
            ui.label("ความเร็วแอนิเมชัน");
            ui.add(egui::Slider::new(&mut self.cfg.timing.speed, 0.5..=2.0).suffix(" x"));
            ui.end_row();
            ui.label("เวลาแสดงผล (วินาที)");
            ui.add(egui::Slider::new(&mut self.cfg.timing.reveal_seconds, 2.0..=15.0));
            ui.end_row();
        });
        ui.label("เคล็ดลับ: เปิดคลิกทะลุแล้วจะลากหน้าต่างไม่ได้ ให้ปิดก่อนแล้วค่อยลากย้าย");
    }

    fn tab_gifts(&mut self, ui: &mut egui::Ui) {
        if self.gift_rows_theme != self.theme() {
            self.sync_edit_buffers();
            self.load_gift_rows();
        }
        let th = self.theme();
        ui.heading(format!("ของขวัญ -> การกระทำ (โหมด{})", match th {
            Theme::Krathong => "ลอยกระทง",
            Theme::Cat => "ให้อาหารแมว",
            Theme::Shrine => "ไหว้พระ",
        }));
        if th == Theme::Krathong {
            ui.label("โหมดลอยกระทงมีตารางของตัวเอง (ถ้าว่างจะใช้ตารางของโหมดไหว้พระ)");
        } else if th == Theme::Cat {
            ui.label("โหมดให้อาหารแมวมีตารางของตัวเอง (ถ้าว่างจะใช้ตารางของโหมดไหว้พระ)");
        }
        ui.label("ใส่ชื่อของขวัญภาษาอังกฤษตามที่ TikTok ใช้ (เช่น Rose, TikTok, Finger Heart) หรือเลข gift id");
        let mut remove = None;
        egui::Grid::new("gifts").num_columns(3).striped(true).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label(egui::RichText::new("ของขวัญ / id").strong());
            ui.label(egui::RichText::new("การกระทำ").strong());
            ui.label("");
            ui.end_row();
            for (i, (name, act)) in self.gift_rows.iter_mut().enumerate() {
                ui.add_sized([220.0, 26.0], egui::TextEdit::singleline(name));
                action_combo(ui, ("g", i), act, th);
                if ui.button("ลบ").clicked() {
                    remove = Some(i);
                }
                ui.end_row();
            }
        });
        if let Some(i) = remove {
            self.gift_rows.remove(i);
        }
        if ui.button("+ เพิ่มของขวัญ").clicked() {
            self.gift_rows.push((String::new(), "pray".into()));
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("ของขวัญอื่น ๆ ที่ไม่อยู่ในตาราง:");
            action_combo(ui, ("def", 0), &mut self.cfg.tiktok.default_action, th);
        });
        ui.horizontal(|ui| {
            ui.label("ส่งแบบคอมโบ เล่นซ้ำสูงสุด");
            ui.add(egui::DragValue::new(&mut self.cfg.tiktok.max_plays_per_gift).range(1..=20));
            ui.label("รอบ");
        });
        self.sync_edit_buffers();
    }

    fn tab_tiers(&mut self, ui: &mut egui::Ui) {
        ui.heading("ระดับความหรูหรา (ตามมูลค่าเพชร)");
        ui.label("มูลค่า = เพชรของของขวัญ × จำนวนที่ส่ง  ยิ่งแพง ตัวละครยิ่งหรู พลุยิ่งใหญ่");
        let desc = ["ชุดธรรมดา ประกายเล็ก", "ผ้าคาดทอง พลุ 2 ลูก", "ชุดทอง พรมทอง พลุ 4 ลูก", "มงกุฎ + ออร่า พลุ 8 ลูก", "ชุดจักรพรรดิ ผู้ติดตามกางฉัตร พลุเต็มฟ้า + แสงระยิบ"];
        egui::Grid::new("tiers").num_columns(4).striped(true).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label(egui::RichText::new("ระดับ").strong());
            ui.label(egui::RichText::new("เพชรขั้นต่ำ").strong());
            ui.label(egui::RichText::new("ชื่อระดับ").strong());
            ui.label(egui::RichText::new("เอฟเฟกต์").strong());
            ui.end_row();
            for (i, t) in self.cfg.tiers.iter_mut().enumerate() {
                ui.label(format!("{}", i + 1));
                ui.add(egui::DragValue::new(&mut t.min_diamonds).range(0..=1_000_000).speed(5.0));
                ui.add_sized([170.0, 26.0], egui::TextEdit::singleline(&mut t.name));
                ui.add(egui::Label::new(egui::RichText::new(desc.get(i).copied().unwrap_or("เหมือนระดับ 5")).size(14.0)).wrap());
                ui.end_row();
            }
        });
        self.cfg.tiers.sort_by_key(|t| t.min_diamonds);
    }

    fn tab_numbers(&mut self, ui: &mut egui::Ui) {
        ui.heading("เลขมงคล");
        ui.horizontal(|ui| {
            ui.label("หัวข้อ");
            ui.text_edit_singleline(&mut self.cfg.numbers.title);
        });
        ui.horizontal(|ui| {
            ui.label("สุ่มเลข:");
            for d in 1..=4u32 {
                let mut on = self.cfg.numbers.digits.contains(&d);
                if ui.checkbox(&mut on, digit_label(d)).changed() {
                    if on {
                        self.cfg.numbers.digits.push(d);
                    } else {
                        self.cfg.numbers.digits.retain(|x| *x != d);
                    }
                    self.cfg.numbers.digits.sort();
                    if self.cfg.numbers.digits.is_empty() {
                        self.cfg.numbers.digits.push(2);
                    }
                }
            }
        });
        ui.separator();
        ui.heading("ลอยกระทง: คำอวยพร");
        ui.horizontal(|ui| {
            ui.label("หัวข้อ");
            ui.text_edit_singleline(&mut self.cfg.krathong.wish_title);
        });
        ui.horizontal(|ui| {
            ui.label("กระทงลอยค้างได้สูงสุด");
            ui.add(egui::DragValue::new(&mut self.cfg.krathong.max_floating).range(1..=60));
            ui.label("ใบ  นาน");
            ui.add(egui::DragValue::new(&mut self.cfg.krathong.float_seconds).range(10.0..=900.0));
            ui.label("วินาที");
        });
        ui.label(format!("คำอวยพร 1 บรรทัด = 1 ข้อ (ตอนนี้ {} ข้อ)", self.blessings_text.lines().filter(|l| !l.trim().is_empty()).count()));
        ui.add(egui::TextEdit::multiline(&mut self.blessings_text).desired_rows(8).desired_width(f32::INFINITY));
        ui.separator();
        ui.heading("เซียมซี");
        ui.horizontal(|ui| {
            ui.label("หัวข้อ");
            ui.text_edit_singleline(&mut self.cfg.siamsi.title);
        });
        ui.label(format!("คำทำนาย 1 บรรทัด = 1 ใบ (ตอนนี้ {} ใบ)", self.fortunes_text.lines().filter(|l| !l.trim().is_empty()).count()));
        ui.add(egui::TextEdit::multiline(&mut self.fortunes_text).desired_rows(14).desired_width(f32::INFINITY));
        ui.separator();
        ui.heading("ให้อาหารแมว: คำขอบคุณ");
        ui.horizontal(|ui| {
            ui.label("หัวข้อ");
            ui.text_edit_singleline(&mut self.cfg.cat.thanks_title);
        });
        ui.label(format!("1 บรรทัด = 1 ข้อความ (ตอนนี้ {} ข้อ)", self.thanks_text.lines().filter(|l| !l.trim().is_empty()).count()));
        ui.add(egui::TextEdit::multiline(&mut self.thanks_text).desired_rows(8).desired_width(f32::INFINITY));
        self.sync_edit_buffers();
    }

    fn tab_test(&mut self, ui: &mut egui::Ui) {
        ui.heading("ทดสอบเอฟเฟกต์");
        ui.horizontal(|ui| {
            ui.label("ระดับ:");
            for i in 0..self.cfg.tiers.len() {
                ui.selectable_value(&mut self.test_tier, i, format!("{} {}", i + 1, self.cfg.tiers[i].name));
            }
        });
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new(self.theme().action_label("pray")).size(18.0)).min_size(Vec2::new(240.0, 48.0))).clicked() {
                let n = test_name(&mut self.rng);
                self.enqueue(Action::Pray, &n, self.tier_min(self.test_tier), self.test_tier);
            }
            if ui.add(egui::Button::new(egui::RichText::new(self.theme().action_label("siamsi")).size(18.0)).min_size(Vec2::new(200.0, 48.0))).clicked() {
                let n = test_name(&mut self.rng);
                self.enqueue(Action::Siamsi, &n, self.tier_min(self.test_tier), self.test_tier);
            }
        });
        if ui.button("ล้างคิว").clicked() {
            self.queue.clear();
            self.cats.clear();
        }
        ui.separator();
        ui.label("ทดสอบผ่าน HTTP (เช่นจาก TikFinity / เบราว์เซอร์):");
        ui.code(format!("http://{}/event?action=pray&user=สมชาย&diamonds=100", self.cfg.http.bind));
        ui.code(format!("http://{}/gift?gift_name=Rose&user=สมชาย&repeat=5", self.cfg.http.bind));
    }

    // ------------------------------------------------------------------ updates
    fn update_banner_ui(&mut self, ui: &mut egui::Ui) {
        let s = self.updater.snapshot();
        if s.available && !s.busy && s.restart.is_none() {
            let tag = s.latest.as_ref().map(|l| l.tag.clone()).unwrap_or_default();
            egui::Frame::new().fill(Color32::from_rgb(120, 80, 10)).corner_radius(8).inner_margin(8).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("มีเวอร์ชันใหม่ {tag} (ตอนนี้ v{})", s.current)).size(17.0).strong().color(Color32::WHITE));
                    if ui.add(egui::Button::new(egui::RichText::new("อัปเดตเลย").size(17.0).strong()).fill(Color32::from_rgb(30, 150, 70))).clicked() {
                        self.updater.apply();
                        self.settings_tab = 7;
                    }
                    if ui.button("ดูรายละเอียด").clicked() {
                        self.settings_tab = 7;
                    }
                });
            });
        }
    }

    fn tab_update(&mut self, ui: &mut egui::Ui) {
        let s = self.updater.snapshot();
        ui.heading("อัปเดตโปรแกรม");
        ui.label(format!("เวอร์ชันที่ใช้อยู่: v{}", s.current));
        match (&s.latest, s.checking) {
            (_, true) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("กำลังตรวจหาอัปเดต...");
                });
            }
            (Some(l), _) => {
                if s.available {
                    ui.label(egui::RichText::new(format!("มีเวอร์ชันใหม่: {}", l.tag)).size(18.0).strong().color(Color32::from_rgb(255, 210, 80)));
                } else {
                    ui.label(egui::RichText::new(format!("เป็นเวอร์ชันล่าสุดแล้ว ({})", l.tag)).color(Color32::from_rgb(120, 220, 120)));
                }
            }
            _ => {}
        }
        ui.horizontal(|ui| {
            if ui.add_enabled(!s.checking && !s.busy, egui::Button::new(egui::RichText::new("ตรวจหาอัปเดต").size(17.0)).min_size(Vec2::new(180.0, 38.0))).clicked() {
                self.updater.check();
            }
            if s.available && ui.add_enabled(!s.busy, egui::Button::new(egui::RichText::new("อัปเดตเลย").size(17.0).strong()).fill(Color32::from_rgb(30, 150, 70)).min_size(Vec2::new(160.0, 38.0))).clicked() {
                self.updater.apply();
            }
            ui.checkbox(&mut self.cfg.update.check_on_start, "ตรวจทุกครั้งที่เปิดโปรแกรม");
        });
        if s.busy || s.restart.is_some() {
            ui.label(&s.phase);
            if let Some((a, b)) = s.progress {
                let frac = if b > 0 { a as f32 / b as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(frac).show_percentage().text(format!("{:.1} / {:.1} MB", a as f32 / 1e6, b as f32 / 1e6)));
            }
        }
        if !s.error.is_empty() {
            ui.label(egui::RichText::new(&s.error).color(Color32::from_rgb(255, 120, 120)));
        }
        if let Some(l) = &s.latest {
            if s.available && !l.body.trim().is_empty() {
                ui.label(egui::RichText::new(format!("มีอะไรใหม่ใน {}", l.tag)).strong());
                egui::ScrollArea::vertical().id_salt("changelog").max_height(150.0).show(ui, |ui| {
                    ui.add(egui::Label::new(egui::RichText::new(l.body.replace('\r', "")).size(14.0)).wrap());
                });
            }
        }
        ui.separator();
        ui.heading("อัปเดตตัวเชื่อม TikTok");
        ui.label("ถ้า TikTok เปลี่ยนระบบแล้วเชื่อมต่อไม่ได้ กดปุ่มนี้เพื่ออัปเดตเฉพาะตัวเชื่อม (tiktok-live-connector) ไม่ต้องรอแอปเวอร์ชันใหม่");
        egui::Grid::new("bridgever").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label("ที่ใช้อยู่");
            ui.label(egui::RichText::new(&s.bridge_current).strong());
            ui.end_row();
            ui.label("ล่าสุด (ไฟล์สำเร็จรูป)");
            ui.label(s.bridge_release.clone().unwrap_or_else(|| if s.bridge_checking { "กำลังตรวจ...".into() } else { "-".into() }));
            ui.end_row();
            ui.label("ล่าสุด (npm)");
            ui.label(s.bridge_npm.clone().unwrap_or_else(|| if s.bridge_checking { "กำลังตรวจ...".into() } else { "-".into() }));
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(!s.bridge_busy, egui::Button::new(egui::RichText::new("อัปเดตตัวเชื่อม TikTok").size(17.0)).min_size(Vec2::new(240.0, 38.0))).clicked() {
                self.updater.update_bridge(self.cfg.tiktok.node_command.clone());
            }
            if s.bridge_busy {
                ui.spinner();
            }
        });
        if !s.bridge_msg.is_empty() {
            ui.label(&s.bridge_msg);
        }
    }

    fn handle_updates(&mut self, ctx: &egui::Context) {
        let (restart, script) = {
            let mut st = self.updater.st.lock().unwrap();
            (st.restart.take(), st.bridge_new_script.take())
        };
        if let Some(script) = script {
            self.bridge.lock().unwrap().script = script;
            self.reconnect();
            self.flash("อัปเดตตัวเชื่อม TikTok แล้ว".into());
        }
        if let Some(exe) = restart {
            self.save_config(ctx);
            // stop the bridge child so the new instance can take over
            restart_bridge(&self.bridge, "", "", &self.cfg.tiktok.node_command, false);
            let mut c = std::process::Command::new(&exe);
            let mut args = vec![];
            let mut it = std::env::args().skip(1);
            while let Some(a) = it.next() {
                if a == "--fake-version" {
                    it.next();
                } else {
                    args.push(a);
                }
            }
            c.args(args);
            match c.spawn() {
                Ok(_) => {
                    eprintln!("[update] เปิดเวอร์ชันใหม่ {}", exe.display());
                    ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
                }
                Err(e) => self.flash(format!("เปิดเวอร์ชันใหม่ไม่ได้: {e} - ปิดแล้วเปิดโปรแกรมใหม่เอง")),
            }
        }
    }

    fn handle_record(&mut self, ctx: &egui::Context) {
        let Some(rec) = &mut self.rec else { return };
        if self.time >= rec.start && self.time <= rec.end {
            let idx = rec.requested;
            rec.requested += 1;
            let path = rec.dir.join(format!("f{idx:05}.png"));
            let saved = rec.saved.clone();
            ctx.request_screenshot(move |img| {
                save_png(&img, &path, false);
                saved.fetch_add(1, Ordering::SeqCst);
            });
        } else if self.time > rec.end && rec.saved.load(Ordering::SeqCst) >= rec.requested {
            ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
        }
    }

    fn handle_shot(&mut self, ctx: &egui::Context) {
        if let Some(shot) = &mut self.shot {
            if shot.done.load(Ordering::SeqCst) {
                shot.frames_after += 1;
                if shot.frames_after > 2 {
                    ctx.send_viewport_cmd_to(ViewportId::ROOT, ViewportCommand::Close);
                }
                return;
            }
            if shot.settings_tab.is_none() && self.time >= shot.at && !shot.requested {
                shot.requested = true;
                let out = shot.out.clone();
                let done = shot.done.clone();
                let preview = shot.preview_bg;
                ctx.request_screenshot(move |img| {
                    save_png(&img, &out, preview);
                    done.store(true, Ordering::SeqCst);
                });
            }
        }
    }
}

impl eframe::App for ShrineApp {
    fn clear_color(&self, _v: &egui::Visuals) -> [f32; 4] {
        match self.cfg.window.background.as_str() {
            "green" => [0.0, 1.0, 0.0, 1.0],
            "scene" => [0.1, 0.05, 0.15, 1.0],
            _ => [0.0, 0.0, 0.0, 0.0],
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.sprites.is_none() {
            self.sprites = Some(load_sprites(&ctx, &self.assets_dir));
        }
        while let Ok(ev) = self.rx.try_recv() {
            self.handle_incoming(ev, &ctx);
        }
        self.handle_keys(&ctx);
        let dt = self.sim_dt.unwrap_or_else(|| ctx.input(|i| i.stable_dt).min(0.1));
        self.update(dt);
        self.apply_window(&ctx);

        // drag to move / right-click or double-click = settings
        let rect = ctx.content_rect();
        let resp = ui.interact(rect, egui::Id::unique("overlay_drag"), egui::Sense::click_and_drag());
        if resp.drag_started_by(egui::PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if resp.secondary_clicked() || resp.double_clicked() {
            self.settings_open = !self.settings_open;
        }
        self.draw(ui);

        if self.settings_open {
            self.settings_ui(&ctx);
        }
        self.handle_shot(&ctx);
        self.handle_record(&ctx);
        self.handle_updates(&ctx);
        self.publish_state();
        ctx.request_repaint();
    }
}


fn action_combo(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, act: &mut String, th: Theme) {
    let label = |a: &str| th.action_label(a);
    egui::ComboBox::from_id_salt(id).selected_text(label(act)).width(160.0).show_ui(ui, |ui| {
        for a in ["pray", "siamsi", "none"] {
            ui.selectable_value(act, a.to_string(), label(a));
        }
    });
}

fn burst_plan(tier: usize, rng: &mut fastrand::Rng) -> Vec<(f32, u8)> {
    let mut v = vec![(0.6, 0u8)];
    let rockets = [0, 2, 4, 8, 18][tier.min(4)];
    let span = [0.0, 1.0, 1.6, 2.6, 4.0][tier.min(4)];
    for i in 0..rockets {
        v.push((0.2 + span * i as f32 / rockets.max(1) as f32 + rng.f32() * 0.2, 1));
    }
    if tier >= 1 {
        v.push((1.2, 0));
    }
    if tier >= 3 {
        v.push((1.0, 2));
    }
    if tier >= 4 {
        v.push((2.2, 2));
        v.push((3.4, 2));
        v.push((2.0, 0));
    }
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    v
}

fn pick_color(rng: &mut fastrand::Rng, tier: usize) -> Color32 {
    let pal: &[Color32] = if tier >= 3 {
        &[GOLD, Color32::from_rgb(255, 240, 180), Color32::from_rgb(255, 90, 90), Color32::from_rgb(255, 160, 40), Color32::from_rgb(250, 120, 220)]
    } else {
        &[GOLD, Color32::from_rgb(255, 90, 120), Color32::from_rgb(90, 200, 255), Color32::from_rgb(140, 255, 140), Color32::from_rgb(220, 140, 255)]
    };
    pal[rng.usize(0..pal.len())]
}

fn hsv(h: f32, s: f32, v: f32) -> Color32 {
    let c = egui::ecolor::Hsva::new(h, s, v, 1.0);
    c.into()
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n).collect::<String>() + "…"
    }
}

fn test_name(rng: &mut fastrand::Rng) -> String {
    const N: [&str; 8] = ["สมชาย", "มะลิ", "น้องฟ้า", "Somsak_99", "พี่ต้น", "Nana", "ใบเตย", "ลุงเฮง"];
    N[rng.usize(0..N.len())].to_string()
}

fn load_sprites(ctx: &egui::Context, assets: &std::path::Path) -> Sprites {
    let load = |name: &str| -> Option<egui::TextureHandle> {
        let p = assets.join("sprites").join(name);
        let img = image::open(&p).ok()?.to_rgba8();
        let size = [img.width() as usize, img.height() as usize];
        let ci = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
        eprintln!("[assets] ใช้รูป {}", p.display());
        Some(ctx.load_texture(name, ci, egui::TextureOptions::LINEAR))
    };
    Sprites { background: load("background.png"), buddha: load("buddha.png"), character: load("character.png") }
}

pub fn save_png(img: &egui::ColorImage, out: &std::path::Path, preview: bool) {
    let [w, h] = img.size;
    let mut buf = image::RgbaImage::new(w as u32, h as u32);
    for (i, px) in img.pixels.iter().enumerate() {
        let [r, g, b, a] = px.to_srgba_unmultiplied();
        buf.put_pixel((i % w) as u32, (i / w) as u32, image::Rgba([r, g, b, a]));
    }
    if let Some(d) = out.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Err(e) = buf.save(out) {
        eprintln!("save {}: {e}", out.display());
    }
    if preview {
        // composite on a checkerboard so transparency is visible
        let mut p = image::RgbaImage::new(w as u32, h as u32);
        for (x, y, px) in buf.enumerate_pixels() {
            let ck = if ((x / 16) + (y / 16)) % 2 == 0 { 70u8 } else { 100u8 };
            let a = px[3] as f32 / 255.0;
            let mix = |c: u8| (c as f32 * a + ck as f32 * (1.0 - a)).round() as u8;
            p.put_pixel(x, y, image::Rgba([mix(px[0]), mix(px[1]), mix(px[2]), 255]));
        }
        let stem = out.file_stem().unwrap().to_string_lossy().to_string();
        let _ = p.save(out.with_file_name(format!("{stem}_checker.png")));
    }
    eprintln!("[shot] saved {}", out.display());
}

/// small down-pointing marker in the sender's colour under a name label (matches the colour tag on their result card)
fn tag_pointer(cx: &Cx, x: f32, y: f32, c: Color32, alpha: f32) {
    if alpha <= 0.01 {
        return;
    }
    let c = with_a(c, alpha);
    cx.poly(&[(x - 11.0, y + 1.0), (x + 11.0, y + 1.0), (x, y + 15.0)], rgba(255, 255, 255, 0.9 * alpha));
    cx.poly(&[(x - 7.0, y + 2.5), (x + 7.0, y + 2.5), (x, y + 11.5)], c);
}

/// TikTok nicknames often carry emoji / decorative symbols the bundled fonts can't draw (they'd show as tofu boxes):
/// drop those, strip control chars and collapse whitespace. Thai, Latin, digits and common punctuation stay.
pub fn display_name(raw: &str) -> String {
    let keep = |c: char| {
        let u = c as u32;
        !(c.is_control()
            || (0x1F000..=0x1FAFF).contains(&u)
            || (0x2600..=0x27BF).contains(&u)
            || (0x2B00..=0x2BFF).contains(&u)
            || (0xFE00..=0xFE0F).contains(&u)
            || (0xE0000..=0xE007F).contains(&u)
            || (0xE000..=0xF8FF).contains(&u)
            || u == 0x200D
            || u == 0x20E3
            || u >= 0xF0000)
    };
    let s: String = raw.chars().filter(|&c| keep(c)).collect();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod name_tests {
    use super::display_name;
    #[test]
    fn strips_emoji_keeps_thai() {
        assert_eq!(display_name("มะลิ🌸 ✨ Nok\u{200D}❤️"), "มะลิ Nok");
        assert_eq!(display_name("🔥🔥"), "");
        assert_eq!(display_name("  สม  ชาย  "), "สม ชาย");
    }
}
