//! Overlay app: event queue, animation timeline, fireworks, settings window.
use crate::config::{digit_label, Config};
use crate::events::{restart_bridge, Action, BridgeCtl, ControlCmd, DirectIn, GiftIn, Incoming};
use crate::scene::*;
use egui::{Align2, Color32, Key, Pos2, Vec2, ViewportBuilder, ViewportCommand, ViewportId, WindowLevel};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

pub struct Show {
    pub action: Action,
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub t: f32,
    pub numbers: Vec<(String, String)>,
    pub stick: usize,
    pub fortune: String,
    pub shirt: Color32,
    pub pants: Color32,
    bursts: Vec<(f32, u8)>, // (time since reveal, kind)
    fired: usize,
    trail_acc: f32,
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
    rng: fastrand::Rng,
    time: f32,
    status: String,
    pub running: bool,
    settings_open: bool,
    settings_tab: usize,
    test_tier: usize,
    applied: Option<Applied>,
    sprites: Option<Sprites>,
    assets_dir: PathBuf,
    shot: Option<ShotPlan>,
    sim_dt: Option<f32>,
    // settings edit buffers
    gift_rows: Vec<(String, String)>,
    fortunes_text: String,
    flash_msg: (String, f32),
    pos_edit: Option<(f32, f32)>,
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
}

impl ShrineApp {
    pub fn new(init: AppInit) -> Self {
        let rng = match init.seed {
            Some(s) => fastrand::Rng::with_seed(s),
            None => fastrand::Rng::new(),
        };
        let sim_dt = init.shot.as_ref().map(|_| 1.0 / 60.0);
        let settings_tab = init.shot.as_ref().and_then(|s| s.settings_tab).unwrap_or(0);
        let mut app = Self {
            gift_rows: init.cfg.gifts.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            fortunes_text: init.cfg.siamsi.fortunes.join("\n"),
            cfg: init.cfg,
            cfg_path: init.cfg_path,
            rx: init.rx,
            bridge: init.bridge,
            queue: VecDeque::new(),
            current: None,
            particles: vec![],
            rng,
            time: 0.0,
            status: String::new(),
            running: !init.start_stopped,
            settings_open: init.open_settings,
            settings_tab,
            test_tier: 0,
            applied: None,
            sprites: None,
            assets_dir: init.assets_dir,
            shot: init.shot,
            sim_dt,
            flash_msg: (String::new(), 0.0),
            pos_edit: None,
        };
        for (a, tier, user) in init.demo {
            let d = app.tier_min(tier).max(1);
            app.enqueue(a, &user, d, tier);
        }
        app
    }

    fn tier_min(&self, tier: usize) -> u64 {
        self.cfg.tiers.get(tier).map(|t| t.min_diamonds).unwrap_or(1)
    }

    fn enqueue(&mut self, action: Action, user: &str, diamonds: u64, tier_override: usize) {
        let tier = if tier_override < 99 { tier_override.min(self.cfg.tiers.len().saturating_sub(1)) } else { self.cfg.tier_for(diamonds) };
        let tier_name = self.cfg.tiers.get(tier).map(|t| t.name.clone()).unwrap_or_default();
        let numbers = self
            .cfg
            .numbers
            .digits
            .iter()
            .map(|&d| {
                let d = d.clamp(1, 6);
                let max = 10u32.pow(d);
                (digit_label(d), format!("{:0width$}", self.rng.u32(0..max), width = d as usize))
            })
            .collect();
        let n = self.cfg.siamsi.fortunes.len().max(1);
        let stick = self.rng.usize(0..n);
        let fortune = self.cfg.siamsi.fortunes.get(stick).cloned().unwrap_or_default();
        let hue = self.rng.f32();
        let shirt = hsv(hue, 0.55, 0.9);
        let pants = hsv((hue + 0.5).fract(), 0.4, 0.35);
        let bursts = burst_plan(tier, &mut self.rng);
        let user = if user.trim().is_empty() { "ผู้ชม".to_string() } else { truncate(user.trim(), 18) };
        self.queue.push_back(Show {
            action,
            user,
            tier,
            tier_name,
            t: 0.0,
            numbers,
            stick: stick + 1,
            fortune,
            shirt,
            pants,
            bursts,
            fired: 0,
            trail_acc: 0.0,
        });
    }

    fn handle_incoming(&mut self, ev: Incoming) {
        match ev {
            Incoming::Status(s) => self.status = s,
            Incoming::Control(c) => match c {
                ControlCmd::ClickThrough(v) => self.cfg.window.click_through = v.unwrap_or(!self.cfg.window.click_through),
                ControlCmd::Background(b) => self.cfg.window.background = b,
                ControlCmd::ClearQueue => self.queue.clear(),
            },
            Incoming::Direct(DirectIn { action, user, diamonds, .. }) => {
                if self.running {
                    self.enqueue(action, &user, diamonds, 99)
                }
            }
            Incoming::Gift(GiftIn { user, gift_id, gift_name, diamonds, repeat }) => {
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

    // timeline (seconds of show-time)
    fn durations(&self, s: &Show) -> (f32, f32, f32, f32) {
        let reveal = self.cfg.timing.reveal_seconds.max(1.5) + s.tier as f32 * 0.5;
        match s.action {
            Action::Pray => (3.0, 2.8, reveal, 1.8),
            Action::Siamsi => (3.0, 4.2, reveal + 0.5, 1.8),
        }
    }

    fn update(&mut self, dt: f32) {
        self.time += dt;
        if self.current.is_none() {
            self.current = self.queue.pop_front();
        }
        let speed = self.cfg.timing.speed;
        let mut finished = false;
        let mut spawn: Vec<(u8, usize)> = vec![];
        let mut trail: Option<(f32, usize)> = None;
        if let Some(s) = &self.current {
            let (w, a, r, o) = self.durations(s);
            let _ = o;
            let _ = (w, a, r);
        }
        if let Some(mut s) = self.current.take() {
            let (w, a, r, o) = self.durations(&s);
            s.t += dt * speed;
            let rt = s.t - w - a;
            while s.fired < s.bursts.len() && rt >= s.bursts[s.fired].0 {
                spawn.push((s.bursts[s.fired].1, s.tier));
                s.fired += 1;
            }
            if s.tier >= 2 && s.t < w {
                s.trail_acc += dt * (s.tier as f32 * 10.0);
                if s.trail_acc >= 1.0 {
                    s.trail_acc = 0.0;
                    trail = Some((lerp(-120.0, PRAY_X, s.t / w), s.tier));
                }
            }
            if s.t >= w + a + r + o {
                finished = true;
            }
            if !finished {
                self.current = Some(s);
            }
        }
        if let Some((x, tier)) = trail {
            let g = self.ground_v();
            for _ in 0..tier {
                let c = if self.rng.bool() { GOLD } else { Color32::WHITE };
                self.particles.push(Particle { pos: Pos2::new(x - 30.0 + self.rng.f32() * 30.0, g - 20.0 - self.rng.f32() * 200.0), vel: Vec2::new(-30.0, -20.0), life: 1.0, max: 1.0, color: c, size: 4.0, kind: 1, tier });
            }
        }
        for (kind, tier) in spawn {
            self.spawn_effect(kind, tier);
        }
        // top tier: sparkle rain during whole show
        if let Some(s) = &self.current {
            if s.tier >= 4 && self.rng.f32() < 0.6 {
                let x = self.rng.f32() * VW;
                self.particles.push(Particle { pos: Pos2::new(x, -10.0), vel: Vec2::new(0.0, 120.0 + self.rng.f32() * 80.0), life: 4.0, max: 4.0, color: GOLD, size: 4.0, kind: 1, tier: 4 });
            }
        }
        // particles
        let mut explode: Vec<(Pos2, usize)> = vec![];
        for p in &mut self.particles {
            p.life -= dt;
            if p.kind == 2 {
                p.vel.y += 600.0 * dt;
                if p.vel.y > -40.0 {
                    p.life = 0.0;
                    explode.push((p.pos, p.tier));
                }
            } else if p.kind == 0 {
                p.vel.y += 160.0 * dt;
                p.vel *= 1.0 - 1.2 * dt;
            } else {
                p.vel *= 1.0 - 0.5 * dt;
            }
            p.pos += p.vel * dt;
        }
        self.particles.retain(|p| p.life > 0.0 && p.pos.y < 4000.0);
        for (pos, tier) in explode {
            self.explode(pos, tier);
        }
        if finished {
            self.current = self.queue.pop_front();
        }
        if self.flash_msg.1 > 0.0 {
            self.flash_msg.1 -= dt;
        }
    }

    fn ground_v(&self) -> f32 {
        let (w, h) = self.cfg.window_size();
        let vh = h / w * VW;
        vh - 110.0_f32.min(vh * 0.12)
    }

    fn spawn_effect(&mut self, kind: u8, tier: usize) {
        let g = self.ground_v();
        match kind {
            // glitter burst around the card
            0 => {
                let n = 30 + tier * 20;
                for _ in 0..n {
                    let a = self.rng.f32() * std::f32::consts::TAU;
                    let sp = 80.0 + self.rng.f32() * (120.0 + tier as f32 * 60.0);
                    let c = pick_color(&mut self.rng, tier);
                    self.particles.push(Particle { pos: Pos2::new(430.0, 230.0), vel: Vec2::new(a.cos(), a.sin()) * sp, life: 1.6, max: 1.6, color: c, size: 4.0 + tier as f32, kind: 1, tier });
                }
            }
            // rocket
            1 => {
                let x = 80.0 + self.rng.f32() * (VW - 160.0);
                // apex height (virtual px) -> launch speed for gravity 600
                let h = (g * (0.45 + self.rng.f32() * 0.3)).min(g - 80.0);
                let vy = -(2.0 * 600.0 * h).sqrt();
                self.particles.push(Particle { pos: Pos2::new(x, g), vel: Vec2::new(self.rng.f32() * 60.0 - 30.0, vy), life: 5.0, max: 5.0, color: GOLD, size: 4.0, kind: 2, tier });
            }
            // gold rain from top
            _ => {
                for _ in 0..(60 + tier * 30) {
                    let x = self.rng.f32() * VW;
                    self.particles.push(Particle { pos: Pos2::new(x, -self.rng.f32() * 200.0), vel: Vec2::new(self.rng.f32() * 30.0 - 15.0, 160.0 + self.rng.f32() * 160.0), life: 3.5, max: 3.5, color: if self.rng.bool() { GOLD } else { Color32::from_rgb(255, 240, 200) }, size: 4.0, kind: 1, tier });
                }
            }
        }
    }

    fn explode(&mut self, pos: Pos2, tier: usize) {
        let n = [50, 70, 90, 140, 220][tier.min(4)];
        let sp = [140.0, 170.0, 210.0, 280.0, 360.0][tier.min(4)];
        let c1 = pick_color(&mut self.rng, tier);
        let c2 = pick_color(&mut self.rng, tier);
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU + self.rng.f32() * 0.1;
            let ring = if tier >= 3 && i % 3 == 0 { 0.55 } else { 1.0 };
            let v = sp * ring * (0.85 + self.rng.f32() * 0.3);
            let life = 1.4 + self.rng.f32() * 0.6 + tier as f32 * 0.15;
            self.particles.push(Particle { pos, vel: Vec2::new(a.cos(), a.sin()) * v, life, max: life, color: if ring < 1.0 { c2 } else { c1 }, size: 3.0 + tier as f32 * 0.6, kind: 0, tier });
        }
        if tier >= 3 {
            for _ in 0..30 {
                let a = self.rng.f32() * std::f32::consts::TAU;
                self.particles.push(Particle { pos, vel: Vec2::new(a.cos(), a.sin()) * sp * 0.4 * self.rng.f32(), life: 2.5, max: 2.5, color: Color32::WHITE, size: 3.0, kind: 1, tier });
            }
        }
    }

    // ------------------------------------------------------------------ drawing
    fn draw(&mut self, ui: &mut egui::Ui) {
        let rect = ui.ctx().content_rect();
        let painter = ui.painter_at(rect);
        let bg = self.cfg.window.background.clone();
        if bg == "green" {
            painter.rect_filled(rect, 0, Color32::from_rgb(0, 255, 0));
        }
        let mut cx = Cx::new(&painter, rect, self.time);
        cx.soft = bg != "green";
        let cx = cx;
        let sprites = self.sprites.as_ref().unwrap();
        if bg == "scene" {
            draw_backdrop(&cx, sprites);
        } else {
            draw_platform(&cx);
        }
        let g = cx.ground();

        let mut shrine_glow = 0.0;
        let mut cyl_on_altar = true;
        if let Some(s) = &self.current {
            let (w, a, r, o) = self.durations(s);
            let t = s.t;
            if s.tier >= 2 {
                let alpha = smooth(t / 0.8) * (1.0 - smooth((t - (w + a + r)) / o));
                draw_carpet(&cx, alpha, s.tier >= 4);
            }
            if t > w && t < w + a + r {
                shrine_glow = smooth((t - w) / 1.0) * if s.action == Action::Pray { 1.0 } else { 0.5 };
            }
            if s.action == Action::Siamsi && t > w + 0.3 && t < w + a + r {
                cyl_on_altar = false;
            }
        }
        draw_shrine(&cx, sprites, shrine_glow, cyl_on_altar);

        if let Some(s) = &self.current {
            self.draw_show(&cx, s);
        }
        draw_particles(&cx, &self.particles);
        if let Some(s) = &self.current {
            if s.tier >= 4 {
                shimmer(&cx, 1.0);
            }
            self.draw_reveal(&cx, s);
        }

        // HUD
        if !self.queue.is_empty() {
            let txt = format!("คิวถัดไป {} คน", self.queue.len());
            let m = cx.measure(&txt, 26.0, true);
            cx.rect(16.0, 16.0, 40.0 + m.x, 62.0, 22.0, rgba(0, 0, 0, 0.55));
            cx.text(28.0, 39.0, Align2::LEFT_CENTER, &txt, 26.0, Color32::WHITE, true);
        }
        if self.cfg.window.show_status && !self.status.is_empty() && self.current.is_none() {
            cx.text_o(VW - 16.0, 24.0, Align2::RIGHT_CENTER, &self.status, 20.0, Color32::WHITE, rgba(0, 0, 0, 0.8), false);
        }
        if !self.running {
            cx.text_o(VW - 16.0, 56.0, Align2::RIGHT_CENTER, "หยุดรับของขวัญชั่วคราว", 22.0, rgba(255, 120, 120, 1.0), rgba(0, 0, 0, 0.8), true);
        }
        if self.cfg.window.show_hint && self.current.is_none() && !self.cfg.window.hint_text.is_empty() {
            let y = (g + cx.vh) / 2.0 + 6.0;
            cx.text_o(VW / 2.0, y, Align2::CENTER_CENTER, &self.cfg.window.hint_text, 28.0, GOLD, rgba(60, 0, 10, 0.95), true);
        }
        if self.flash_msg.1 > 0.0 {
            cx.text_o(VW / 2.0, 30.0, Align2::CENTER_CENTER, &self.flash_msg.0, 24.0, Color32::WHITE, rgba(0, 0, 0, 0.9), true);
        }
        let _ = Vec2::ZERO;
        let _ = g;
    }

    fn draw_show(&self, cx: &Cx, s: &Show) {
        let sprites = self.sprites.as_ref().unwrap();
        let (w, a, r, o) = self.durations(s);
        let t = s.t;
        let g = cx.ground();
        let mut pose = Pose { walk: 0.0, moving: false, wai: 0.0, bow: 0.0, kneel: 0.0, hold_cyl: None, facing: 1.0 };
        let x;
        if t < w {
            x = lerp(-120.0, PRAY_X, t / w);
            pose.walk = t * 9.0;
            pose.moving = true;
        } else if t < w + a {
            x = PRAY_X;
            let at = t - w;
            match s.action {
                Action::Pray => {
                    pose.kneel = smooth(at / 0.5);
                    pose.wai = smooth((at - 0.3) / 1.0);
                    let b = ((at - 1.2) / 1.5).clamp(0.0, 1.0);
                    pose.bow = (b * std::f32::consts::PI * 2.0).sin().abs() * 0.8;
                }
                Action::Siamsi => {
                    pose.kneel = smooth(at / 0.5);
                    if at < 3.0 {
                        pose.hold_cyl = Some(if at > 0.5 { ((at - 0.5) * 22.0).sin() * 0.28 } else { 0.0 });
                        pose.bow = 0.25;
                    } else {
                        pose.hold_cyl = Some(0.15);
                    }
                }
            }
        } else if t < w + a + r {
            x = PRAY_X;
            pose.kneel = 1.0;
            match s.action {
                Action::Pray => pose.wai = 1.0,
                Action::Siamsi => pose.hold_cyl = Some(0.15),
            }
        } else {
            let ot = t - w - a - r;
            x = lerp(PRAY_X, -160.0, ot / o);
            pose.walk = ot * 9.0;
            pose.moving = true;
            pose.facing = -1.0;
        }
        // entourage
        if s.tier >= 4 {
            for (i, off) in [150.0, 270.0].iter().enumerate() {
                let mut p2 = Pose { walk: pose.walk + i as f32 * 1.5, moving: pose.moving, wai: 0.0, bow: 0.0, kneel: 0.0, hold_cyl: None, facing: pose.facing };
                if !pose.moving {
                    p2.wai = pose.wai * 0.0;
                }
                let ax = x - off * pose.facing;
                draw_attendant(cx, sprites, ax, &p2);
            }
        }
        let look = Look { tier: s.tier, shirt: s.shirt, pants: s.pants, scale: 1.15 };
        draw_character(cx, sprites, x, &pose, &look);

        // stick falling (siamsi)
        if s.action == Action::Siamsi && t > w + 3.0 {
            let ft = ((t - w - 3.0) / 1.0).clamp(0.0, 1.0);
            let sx = lerp(x + 95.0, x + 170.0, ft);
            let sy = lerp(g - 230.0, g - 12.0, ft) - (ft * std::f32::consts::PI).sin() * 70.0;
            let ang = ft * 4.5;
            let (sn, cs) = ang.sin_cos();
            let len = 60.0;
            cx.line((sx - cs * len, sy - sn * len), (sx + cs * len, sy + sn * len), 7.0, rgba(225, 180, 100, 1.0));
            cx.circle(sx + cs * len, sy + sn * len, 5.0, RED);
            if ft >= 1.0 {
                cx.glow(sx, sy, 70.0, GOLD, 0.5);
                cx.text_o(sx, sy - 40.0, Align2::CENTER_CENTER, &s.stick.to_string(), 40.0, RED, Color32::WHITE, true);
            }
        }
        // name banner
        let head_top = g - if pose.kneel > 0.5 { 210.0 } else { 260.0 } - if s.tier >= 3 { 45.0 } else { 0.0 };
        draw_name_banner(cx, x, head_top, &s.user, s.tier, if s.tier >= 2 { &s.tier_name } else { "" });
    }

    fn draw_reveal(&self, cx: &Cx, s: &Show) {
        let (w, a, r, _o) = self.durations(s);
        let rt = s.t - w - a;
        if rt < 0.0 || rt > r {
            return;
        }
        let alpha = smooth(rt / 0.35) * (1.0 - smooth((rt - (r - 0.5)) / 0.5));
        let pop = ease_out_back(rt / 0.45);
        let cxx = 430.0;
        let top = 30.0 + (1.0 - pop) * 40.0;
        let tier = s.tier;
        let border = if tier >= 2 { GOLD } else { rgba(255, 220, 150, 1.0) };
        match s.action {
            Action::Pray => {
                let rows = s.numbers.len() as f32;
                let hgt = 110.0 + rows * 118.0;
                let wdt = 600.0 + tier as f32 * 20.0;
                let (x0, x1) = (cxx - wdt / 2.0, cxx + wdt / 2.0);
                if tier >= 3 {
                    cx.glow(cxx, top + hgt / 2.0, wdt * 0.8, GOLD, 0.45 * alpha);
                }
                cx.rect(x0 + 8.0, top + 10.0, x1 + 8.0, top + hgt + 10.0, 26.0, rgba(0, 0, 0, 0.35 * alpha));
                cx.vgradient(x0, top, x1, top + hgt, rgba(150, 20, 35, 0.96 * alpha), rgba(90, 8, 22, 0.96 * alpha));
                cx.rect_line(x0, top, x1, top + hgt, 18.0, 5.0 + tier as f32, with_a(border, alpha));
                cx.rect_line(x0 + 12.0, top + 12.0, x1 - 12.0, top + hgt - 12.0, 12.0, 2.0, with_a(border, 0.6 * alpha));
                let title = format!("{} ของ {}", self.cfg.numbers.title, s.user);
                cx.text_o(cxx, top + 55.0, Align2::CENTER_CENTER, &title, 38.0, with_a(GOLD, alpha), rgba(60, 0, 0, alpha), true);
                for (i, (label, num)) in s.numbers.iter().enumerate() {
                    let appear = 0.5 + i as f32 * 0.7;
                    let y = top + 110.0 + i as f32 * 118.0 + 55.0;
                    cx.text(x0 + 50.0, y, Align2::LEFT_CENTER, label, 32.0, rgba(255, 235, 200, alpha), true);
                    let lt = rt - appear;
                    let shown: String = if lt < 0.6 {
                        // rolling digits
                        let seed = ((self.time * 30.0) as u32).wrapping_mul(2654435761).wrapping_add(i as u32 * 97);
                        num.chars().enumerate().map(|(j, _)| char::from(b'0' + ((seed >> (j * 3)) % 10) as u8)).collect()
                    } else {
                        num.clone()
                    };
                    let sc = if lt < 0.6 { 0.9 } else { ease_out_back((lt - 0.6) / 0.35) };
                    let size = (86.0 + tier as f32 * 6.0) * sc.max(0.1);
                    if lt >= 0.6 {
                        cx.glow(cxx + 90.0, y, 120.0, GOLD, 0.35 * alpha);
                    }
                    let spaced: String = shown.chars().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
                    cx.text_o(cxx + 90.0, y, Align2::CENTER_CENTER, &spaced, size, with_a(Color32::from_rgb(255, 225, 90), alpha), rgba(80, 0, 0, alpha), true);
                }
            }
            Action::Siamsi => {
                let wdt = 640.0 + tier as f32 * 16.0;
                let (x0, x1) = (cxx - wdt / 2.0, cxx + wdt / 2.0);
                let hgt = 400.0;
                if tier >= 3 {
                    cx.glow(cxx, top + hgt / 2.0, wdt * 0.8, GOLD, 0.45 * alpha);
                }
                cx.rect(x0 + 8.0, top + 10.0, x1 + 8.0, top + hgt + 10.0, 20.0, rgba(0, 0, 0, 0.35 * alpha));
                cx.vgradient(x0, top, x1, top + hgt, rgba(255, 246, 220, 0.97 * alpha), rgba(245, 222, 170, 0.97 * alpha));
                cx.rect_line(x0, top, x1, top + hgt, 16.0, 5.0 + tier as f32, with_a(if tier >= 2 { GOLD_D } else { RED }, alpha));
                // scroll rods
                cx.rect(x0 - 14.0, top - 8.0, x1 + 14.0, top + 14.0, 10.0, rgba(140, 20, 30, alpha));
                cx.rect(x0 - 14.0, top + hgt - 14.0, x1 + 14.0, top + hgt + 8.0, 10.0, rgba(140, 20, 30, alpha));
                let title = format!("{} {}", self.cfg.siamsi.title, s.stick);
                cx.text_o(cxx, top + 75.0, Align2::CENTER_CENTER, &title, 56.0 * pop.max(0.2), rgba(170, 10, 30, alpha), rgba(255, 255, 255, alpha), true);
                cx.line((x0 + 60.0, top + 125.0), (x1 - 60.0, top + 125.0), 3.0, with_a(GOLD_D, alpha));
                let ta = alpha * smooth((rt - 0.5) / 0.6);
                cx.text_wrapped(cxx, top + 150.0, &s.fortune, 36.0, wdt - 100.0, rgba(70, 30, 10, ta), true);
                cx.text(cxx, top + hgt - 40.0, Align2::CENTER_CENTER, &format!("คำทำนายของ {}", s.user), 26.0, rgba(140, 60, 20, ta), false);
            }
        }
    }

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
            } else if k == Key::Delete {
                self.queue.clear();
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
        self.cfg.gifts = self.gift_rows.iter().filter(|(k, _)| !k.trim().is_empty()).map(|(k, v)| (k.trim().to_string(), v.clone())).collect();
        let f: Vec<String> = self.fortunes_text.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
        if !f.is_empty() {
            self.cfg.siamsi.fortunes = f;
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
        let tabs = ["เริ่ม/หยุด", "TikTok", "หน้าจอ", "ของขวัญ", "ระดับ", "เลข/เซียมซี", "ทดสอบ"];
        ui.horizontal_wrapped(|ui| {
            for (i, t) in tabs.iter().enumerate() {
                if ui.selectable_label(self.settings_tab == i, egui::RichText::new(*t).size(17.0)).clicked() {
                    self.settings_tab = i;
                }
            }
        });
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
                _ => self.tab_test(ui),
            }
        });
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button(egui::RichText::new("บันทึกลง config.toml").size(16.0)).clicked() {
                self.save_config(ctx);
            }
            if ui.button(egui::RichText::new("โหลดค่าใหม่").size(16.0)).clicked() {
                self.cfg = Config::load(&self.cfg_path);
                self.gift_rows = self.cfg.gifts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                self.fortunes_text = self.cfg.siamsi.fortunes.join("\n");
                self.flash("โหลดการตั้งค่าใหม่แล้ว".into());
            }
            if self.flash_msg.1 > 0.0 {
                ui.label(egui::RichText::new(&self.flash_msg.0).color(Color32::from_rgb(120, 220, 120)));
            }
        });
    }

    fn tab_main(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
            ui.label(egui::RichText::new(format!("คิว: {} คน   |   กำลังเล่น: {}", self.queue.len(), self.current.as_ref().map(|s| s.user.as_str()).unwrap_or("-"))).size(16.0));
        });
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(egui::RichText::new("ปุ่มลัด (คลิกที่หน้าต่างเอฟเฟกต์ก่อน)").strong());
            ui.label("1 = ไหว้พระขอเลข   2 = เซียมซี   (Shift+1..5 เลือกระดับทดสอบ)");
            ui.label("Q W E R T = ไหว้พระ ระดับ 1-5   |   A S D F G = เซียมซี ระดับ 1-5");
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
                ui.radio_value(&mut self.cfg.window.background, "scene".to_string(), "ฉากวัดเต็มพื้นหลัง");
            });
            ui.end_row();
            ui.label("ลอยอยู่บนสุดเสมอ");
            ui.checkbox(&mut self.cfg.window.always_on_top, "");
            ui.end_row();
            ui.label("คลิกทะลุหน้าต่าง");
            ui.checkbox(&mut self.cfg.window.click_through, "(F8 สลับ)");
            ui.end_row();
            ui.label("แสดงคำแนะนำ");
            ui.checkbox(&mut self.cfg.window.show_hint, "");
            ui.end_row();
            ui.label("ข้อความคำแนะนำ");
            ui.add(egui::TextEdit::singleline(&mut self.cfg.window.hint_text).desired_width(330.0));
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
        ui.heading("ของขวัญ -> การกระทำ");
        ui.label("ใส่ชื่อของขวัญภาษาอังกฤษตามที่ TikTok ใช้ (เช่น Rose, TikTok, Finger Heart) หรือเลข gift id");
        let mut remove = None;
        egui::Grid::new("gifts").num_columns(3).striped(true).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label(egui::RichText::new("ของขวัญ / id").strong());
            ui.label(egui::RichText::new("การกระทำ").strong());
            ui.label("");
            ui.end_row();
            for (i, (name, act)) in self.gift_rows.iter_mut().enumerate() {
                ui.add_sized([220.0, 26.0], egui::TextEdit::singleline(name));
                action_combo(ui, ("g", i), act);
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
            action_combo(ui, ("def", 0), &mut self.cfg.tiktok.default_action);
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
        ui.heading("เซียมซี");
        ui.horizontal(|ui| {
            ui.label("หัวข้อ");
            ui.text_edit_singleline(&mut self.cfg.siamsi.title);
        });
        ui.label(format!("คำทำนาย 1 บรรทัด = 1 ใบ (ตอนนี้ {} ใบ)", self.fortunes_text.lines().filter(|l| !l.trim().is_empty()).count()));
        ui.add(egui::TextEdit::multiline(&mut self.fortunes_text).desired_rows(14).desired_width(f32::INFINITY));
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
            if ui.add(egui::Button::new(egui::RichText::new("ทดสอบ ไหว้พระขอเลข").size(18.0)).min_size(Vec2::new(240.0, 48.0))).clicked() {
                let n = test_name(&mut self.rng);
                self.enqueue(Action::Pray, &n, self.tier_min(self.test_tier), self.test_tier);
            }
            if ui.add(egui::Button::new(egui::RichText::new("ทดสอบ เซียมซี").size(18.0)).min_size(Vec2::new(200.0, 48.0))).clicked() {
                let n = test_name(&mut self.rng);
                self.enqueue(Action::Siamsi, &n, self.tier_min(self.test_tier), self.test_tier);
            }
        });
        if ui.button("ล้างคิว").clicked() {
            self.queue.clear();
        }
        ui.separator();
        ui.label("ทดสอบผ่าน HTTP (เช่นจาก TikFinity / เบราว์เซอร์):");
        ui.code(format!("http://{}/event?action=pray&user=สมชาย&diamonds=100", self.cfg.http.bind));
        ui.code(format!("http://{}/gift?gift_name=Rose&user=สมชาย&repeat=5", self.cfg.http.bind));
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
            self.handle_incoming(ev);
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
        ctx.request_repaint();
    }
}

fn action_combo(ui: &mut egui::Ui, id: impl std::hash::Hash + std::fmt::Debug, act: &mut String) {
    let label = |a: &str| match a {
        "pray" => "ไหว้พระขอเลข",
        "siamsi" => "เสี่ยงเซียมซี",
        _ => "ไม่ทำอะไร",
    };
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
