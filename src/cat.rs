//! โหมดให้อาหารแมว: มุมอบอุ่น แมวเดินเข้าคิว กินตามระดับของขวัญ แล้วขึ้นการ์ดขอบคุณ (ไม่มีเลขมงคล)
use crate::events::Action;
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

pub const BOWL_X: f32 = 760.0;
pub const STAND_OFF: f32 = 78.0;
pub const SLOT_GAP: f32 = 148.0;
pub const OFF_X: f32 = -180.0;
pub const POUR: f32 = 1.15;
pub const EAT: f32 = 2.25;
pub const LEAVE: f32 = 1.65;
pub const WALK_MAX: f32 = 2.4;

pub fn floor_y(vh: f32) -> f32 {
    (vh - 96.0_f32.min(vh * 0.14)).max(vh * 0.62)
}
pub fn stand_x() -> f32 {
    BOWL_X - STAND_OFF
}
pub fn slot_x(k: usize) -> f32 {
    if k == 0 {
        stand_x()
    } else {
        stand_x() - k as f32 * SLOT_GAP
    }
}

/// Thai (and general) combining marks shouldn't count toward the visible name length.
pub fn is_combining(c: char) -> bool {
    let u = c as u32;
    matches!(u, 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E)
        || (0x0300..=0x036F).contains(&u)
        || (0x1DC0..=0x1DFF).contains(&u)
        || (0x20D0..=0x20FF).contains(&u)
        || (0xFE20..=0xFE2F).contains(&u)
}

/// Visible letter/digit units (combining marks excluded).
pub fn visible_units(name: &str) -> usize {
    name.chars().filter(|c| !is_combining(*c)).count()
}

/// Shorten a display name so a tag stays narrow (waiting cats use a tighter cap).
pub fn ellipsize_name(name: &str, max_units: usize) -> String {
    let max_units = max_units.max(1);
    if visible_units(name) <= max_units {
        return name.to_string();
    }
    let keep = max_units.saturating_sub(1);
    let mut out = String::new();
    let mut n = 0;
    for c in name.chars() {
        if !is_combining(c) {
            if n >= keep {
                break;
            }
            n += 1;
        }
        out.push(c);
    }
    out.push('…');
    out
}

/// Estimated half-width of a name tag in virtual pixels (no Painter needed).
/// `front` tags may include a tier ribbon; waiting tags are compact name-only pills.
pub fn tag_half_width(name: &str, front: bool, tier: usize) -> f32 {
    if front {
        let shown = ellipsize_name(name, 14);
        let size = [26.0, 28.0, 32.0, 36.0, 42.0][tier.min(4)];
        let w = visible_units(&shown) as f32 * size * 0.95 + 36.0 + tier as f32 * 10.0;
        let ribbon = if tier >= 3 {
            40.0 + tier as f32 * 8.0
        } else {
            0.0
        };
        w / 2.0 + ribbon
    } else {
        let shown = ellipsize_name(name, 8);
        let size = 24.0;
        visible_units(&shown) as f32 * size * 0.95 / 2.0 + 22.0
    }
}

/// x targets for the visible queue: front at the bowl, others spaced so tags never overlap.
pub fn queue_targets(names: &[&str], tiers: &[usize], max_visible: usize) -> Vec<f32> {
    let n = names.len();
    let cap = max_visible.clamp(1, 12);
    let mut xs = vec![OFF_X; n];
    if n == 0 {
        return xs;
    }
    xs[0] = stand_x();
    for i in 1..n {
        if i >= cap {
            xs[i] = OFF_X;
            continue;
        }
        let prev_half = tag_half_width(
            names[i - 1],
            i - 1 == 0,
            tiers.get(i - 1).copied().unwrap_or(0),
        );
        let this_half = tag_half_width(names[i], false, tiers.get(i).copied().unwrap_or(0));
        let gap = (prev_half + this_half + 20.0).max(130.0);
        xs[i] = (xs[i - 1] - gap).max(70.0);
    }
    xs
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Treat {
    Kibble,
    Can,
    Fish,
    Sashimi,
    Feast,
}
pub fn treat_for(tier: usize) -> Treat {
    [
        Treat::Kibble,
        Treat::Can,
        Treat::Fish,
        Treat::Sashimi,
        Treat::Feast,
    ][tier.min(4)]
}
impl Treat {
    pub fn label(self) -> &'static str {
        match self {
            Treat::Kibble => "อาหารเม็ด",
            Treat::Can => "อาหารกระป๋อง",
            Treat::Fish => "ปลาย่าง",
            Treat::Sashimi => "ซาชิมิ",
            Treat::Feast => "สำรับปลาหลวง",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CatLook {
    pub fur: Color32,
    pub fur_dark: Color32,
    pub fur_light: Color32,
    pub eye: Color32,
    pub pattern: u8, // 0 solid, 1 socks, 2 tabby, 3 calico, 4 tuxedo, 5 siamese
    pub fluff: f32,
    pub scale: f32,
    pub seed: f32,
    pub collar: bool,
    pub bow: bool,
    pub jewels: bool,
    pub crown: bool,
}

pub fn look_for(name: &str, tier: usize) -> CatLook {
    let mut h: u64 = 1469598103934665603;
    for b in name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    let pals: [(Color32, Color32, Color32, Color32); 8] = [
        (
            rgba(232, 168, 96, 1.0),
            rgba(186, 118, 58, 1.0),
            rgba(250, 214, 160, 1.0),
            rgba(80, 160, 70, 1.0),
        ), // orange
        (
            rgba(236, 228, 214, 1.0),
            rgba(196, 184, 164, 1.0),
            rgba(255, 250, 240, 1.0),
            rgba(90, 150, 200, 1.0),
        ), // cream
        (
            rgba(160, 164, 172, 1.0),
            rgba(110, 114, 124, 1.0),
            rgba(210, 214, 220, 1.0),
            rgba(90, 190, 140, 1.0),
        ), // gray
        (
            rgba(46, 42, 44, 1.0),
            rgba(24, 22, 24, 1.0),
            rgba(90, 84, 86, 1.0),
            rgba(230, 200, 80, 1.0),
        ), // black
        (
            rgba(248, 246, 240, 1.0),
            rgba(210, 206, 198, 1.0),
            rgba(255, 255, 255, 1.0),
            rgba(80, 140, 210, 1.0),
        ), // white
        (
            rgba(200, 122, 72, 1.0),
            rgba(150, 78, 44, 1.0),
            rgba(240, 190, 140, 1.0),
            rgba(70, 150, 90, 1.0),
        ), // ginger
        (
            rgba(120, 96, 80, 1.0),
            rgba(80, 58, 46, 1.0),
            rgba(186, 160, 130, 1.0),
            rgba(200, 160, 60, 1.0),
        ), // brown
        (
            rgba(240, 200, 150, 1.0),
            rgba(120, 80, 50, 1.0),
            rgba(255, 230, 190, 1.0),
            rgba(90, 90, 160, 1.0),
        ), // siamese-ish
    ];
    let (fur, fur_dark, fur_light, eye) = pals[(h % pals.len() as u64) as usize];
    let pattern = if (h / 8).is_multiple_of(11) {
        5
    } else {
        ((h / 3) % 5) as u8
    };
    let t = tier.min(4);
    CatLook {
        fur,
        fur_dark,
        fur_light,
        eye,
        pattern,
        fluff: 0.85 + t as f32 * 0.18,
        scale: 0.88 + t as f32 * 0.07,
        seed: ((h >> 11) % 10000) as f32 / 10000.0,
        collar: t >= 1,
        bow: t >= 2,
        jewels: t >= 3,
        crown: t >= 4,
    }
}

#[derive(Clone)]
pub struct CatShow {
    pub action: Action,
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub text: String,
    pub look: CatLook,
    pub t: f32,
    pub qage: f32,
    pub x: f32,
    pub from_x: f32,
    pub target_x: f32,
    pub front: bool,
    pub done: bool,
    walk_dur: f32,
    poured: bool,
    burst_at: f32,
}

impl CatShow {
    pub fn reveal_end(&self, reveal: f32) -> f32 {
        self.walk_dur + POUR + EAT + reveal
    }
    pub fn total(&self, reveal: f32) -> f32 {
        self.reveal_end(reveal) + LEAVE
    }
    pub fn phase(&self, reveal: f32) -> &'static str {
        let t = self.t;
        let w = self.walk_dur;
        if !self.front {
            "wait"
        } else if t < w {
            "walk"
        } else if t < w + POUR {
            "pour"
        } else if t < w + POUR + EAT {
            "eat"
        } else if t < w + POUR + EAT + reveal {
            "card"
        } else {
            "leave"
        }
    }
}

pub struct CatIn {
    pub action: Action,
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub text: String,
}

struct Fx {
    kind: u8, // 0 heart, 1 food, 2 sparkle, 3 burst, 4 mote
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    size: f32,
    rot: f32,
    col: Color32,
    _tier: usize,
}

pub struct Stage {
    shows: VecDeque<CatShow>,
    particles: Vec<Fx>,
    pub bowl_food: f32,
    pub bowl_treat: Treat,
    floor: f32,
    time: f32,
    rng: fastrand::Rng,
    pub started: u64,
    pub finished: u64,
}

impl Stage {
    pub fn new() -> Self {
        Self::with_seed(0xC47u64)
    }
    pub fn with_seed(seed: u64) -> Self {
        Self {
            shows: VecDeque::new(),
            particles: Vec::new(),
            bowl_food: 0.12,
            bowl_treat: Treat::Kibble,
            floor: 430.0,
            time: 0.0,
            rng: fastrand::Rng::with_seed(seed),
            started: 0,
            finished: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.shows.len()
    }
    pub fn busy(&self) -> bool {
        !self.shows.is_empty() || self.particles.iter().any(|p| p.kind != 4)
    }
    pub fn clear(&mut self) {
        self.shows.clear();
        self.particles.retain(|p| p.kind == 4);
    }
    pub fn front_user(&self) -> Option<&str> {
        self.shows.front().map(|s| s.user.as_str())
    }
    pub fn waiting_names(&self) -> Vec<&str> {
        self.shows.iter().skip(1).map(|s| s.user.as_str()).collect()
    }
    pub fn shows(&self) -> impl Iterator<Item = &CatShow> {
        self.shows.iter()
    }
    pub fn visible(&self, max_visible: usize) -> Vec<&CatShow> {
        let cap = max_visible.clamp(1, 12);
        self.shows
            .iter()
            .take(cap)
            .filter(|s| s.x > OFF_X + 20.0)
            .collect()
    }
    pub fn enqueue(&mut self, input: CatIn) {
        let look = look_for(&input.user, input.tier);
        self.shows.push_back(CatShow {
            action: input.action,
            user: input.user,
            tier: input.tier,
            tier_name: input.tier_name,
            text: input.text,
            look,
            t: 0.0,
            qage: 0.0,
            x: OFF_X,
            from_x: OFF_X,
            target_x: OFF_X,
            front: false,
            done: false,
            walk_dur: WALK_MAX,
            poured: false,
            burst_at: -1.0,
        });
    }

    pub fn tick(&mut self, dt: f32, max_visible: usize, reveal_base: f32, vh: f32) {
        self.time += dt;
        self.floor = floor_y(vh);
        while self.shows.front().map(|s| s.done).unwrap_or(false) {
            self.shows.pop_front();
            self.finished += 1;
        }
        let cap = max_visible.clamp(1, 12);
        let n = self.shows.len();
        let names: Vec<&str> = self.shows.iter().map(|s| s.user.as_str()).collect();
        let tiers: Vec<usize> = self.shows.iter().map(|s| s.tier).collect();
        let targets = queue_targets(&names, &tiers, cap);
        for i in 0..n {
            self.shows[i].target_x = targets[i];
            let reveal = reveal_base.max(1.5) + self.shows[i].tier as f32 * 0.5;
            if i == 0 {
                if !self.shows[i].front {
                    self.shows[i].front = true;
                    self.shows[i].from_x = self.shows[i].x;
                    let dist = (stand_x() - self.shows[i].x).abs();
                    self.shows[i].walk_dur = (dist / 290.0).clamp(0.55, WALK_MAX);
                    self.shows[i].t = 0.0;
                    self.started += 1;
                }
                self.tick_front(i, dt, reveal);
            } else {
                self.shows[i].front = false;
                self.shows[i].t = 0.0;
                self.approach(i, targets[i], dt, 260.0);
            }
            self.shows[i].qage += dt;
        }
        // ambient motes stay few
        self.step_fx(dt);
        if self.bowl_food > 0.16 && self.shows.is_empty() {
            self.bowl_food = (self.bowl_food - dt * 0.08).max(0.12);
        }
    }

    fn approach(&mut self, i: usize, target: f32, dt: f32, speed: f32) {
        let s = &mut self.shows[i];
        let d = target - s.x;
        if d.abs() < 1.5 {
            s.x = target;
            return;
        }
        let step = speed
            * dt
            * if d.abs() < 80.0 {
                0.55 + 0.45 * (d.abs() / 80.0)
            } else {
                1.0
            };
        s.x += d.signum() * step.min(d.abs());
    }

    fn tick_front(&mut self, i: usize, dt: f32, reveal: f32) {
        let w = self.shows[i].walk_dur;
        self.shows[i].t += dt;
        let t = self.shows[i].t;
        let stand = stand_x();
        if t < w {
            let u = ease_in_out(t / w);
            self.shows[i].x = lerp(self.shows[i].from_x, stand, u);
        } else if t < w + POUR + EAT + reveal {
            self.shows[i].x = stand;
        } else {
            let u = ease_in_out(((t - w - POUR - EAT - reveal) / LEAVE).clamp(0.0, 1.0));
            self.shows[i].x = lerp(stand, VW + 200.0, u);
        }
        // pour food
        if t >= w && t < w + POUR {
            if !self.shows[i].poured {
                self.shows[i].poured = true;
                self.bowl_treat = treat_for(self.shows[i].tier);
            }
            let fill = 0.2 + 0.75 * smooth((t - w) / POUR);
            self.bowl_food = self.bowl_food.max(fill);
            self.spawn_food(dt, self.shows[i].tier);
        }
        if t >= w + POUR && t < w + POUR + EAT {
            self.spawn_hearts(i, dt);
            self.bowl_food = (self.bowl_food - dt * 0.12).max(0.18);
        }
        if t >= w + POUR + EAT && self.shows[i].burst_at < 0.0 {
            self.shows[i].burst_at = t;
            let tier = self.shows[i].tier;
            self.spawn_celebrate(tier);
        }
        if t >= self.shows[i].total(reveal) {
            self.shows[i].done = true;
        }
    }

    fn spawn_food(&mut self, dt: f32, tier: usize) {
        let rate = 10.0 + tier as f32 * 6.0;
        if self.rng.f32() < rate * dt {
            let treat = treat_for(tier);
            let col = food_color(treat, self.rng.f32());
            let x = BOWL_X + (self.rng.f32() - 0.5) * 36.0;
            let y = self.floor - 160.0 - self.rng.f32() * 40.0;
            self.particles.push(Fx {
                kind: 1,
                x,
                y,
                vx: (self.rng.f32() - 0.5) * 30.0,
                vy: 80.0 + self.rng.f32() * 90.0,
                life: 0.7,
                max: 0.7,
                size: 5.0 + tier as f32 * 1.2,
                rot: self.rng.f32() * TAU,
                col,
                _tier: tier,
            });
        }
    }

    fn spawn_hearts(&mut self, i: usize, dt: f32) {
        let tier = self.shows[i].tier;
        let x = self.shows[i].x;
        let rate = 3.0 + tier as f32 * 1.8;
        if self.rng.f32() < rate * dt {
            let g = self.floor;
            self.particles.push(Fx {
                kind: 0,
                x: x + 20.0 + (self.rng.f32() - 0.5) * 40.0,
                y: g - 70.0 - self.rng.f32() * 20.0,
                vx: (self.rng.f32() - 0.5) * 18.0,
                vy: -(40.0 + self.rng.f32() * 30.0),
                life: 1.4 + self.rng.f32() * 0.5,
                max: 1.8,
                size: 8.0 + tier as f32 * 2.0,
                rot: self.rng.f32() * TAU,
                col: if self.rng.bool() {
                    rgba(255, 110, 150, 1.0)
                } else {
                    rgba(255, 160, 190, 1.0)
                },
                _tier: tier,
            });
        }
        if tier >= 2 && self.rng.f32() < (2.0 + tier as f32) * dt {
            self.particles.push(Fx {
                kind: 2,
                x: x + (self.rng.f32() - 0.5) * 90.0,
                y: self.floor - 80.0 - self.rng.f32() * 80.0,
                vx: 0.0,
                vy: -12.0,
                life: 0.7,
                max: 0.7,
                size: 6.0 + self.rng.f32() * 5.0,
                rot: 0.0,
                col: if tier >= 4 {
                    GOLD
                } else {
                    rgba(255, 230, 140, 1.0)
                },
                _tier: tier,
            });
        }
    }

    fn spawn_celebrate(&mut self, tier: usize) {
        let n = [8, 14, 22, 36, 70][tier.min(4)];
        let g = self.floor;
        for _ in 0..n {
            let a = self.rng.f32() * TAU;
            let sp = 60.0 + self.rng.f32() * (80.0 + tier as f32 * 50.0);
            let col = if tier >= 3 {
                [
                    GOLD,
                    rgba(255, 120, 160, 1.0),
                    rgba(255, 240, 180, 1.0),
                    rgba(255, 180, 80, 1.0),
                ][self.rng.usize(0..4)]
            } else {
                [
                    rgba(255, 140, 170, 1.0),
                    rgba(255, 220, 140, 1.0),
                    rgba(180, 220, 255, 1.0),
                ][self.rng.usize(0..3)]
            };
            self.particles.push(Fx {
                kind: if tier >= 3 { 3 } else { 2 },
                x: BOWL_X,
                y: g - 180.0,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 30.0,
                life: 1.1 + self.rng.f32() * 0.5,
                max: 1.6,
                size: 4.0 + tier as f32 * 1.4,
                rot: self.rng.f32() * TAU,
                col,
                _tier: tier,
            });
        }
        if tier >= 4 {
            for k in 0..5 {
                let a = k as f32 / 5.0 * TAU + 0.4;
                self.particles.push(Fx {
                    kind: 3,
                    x: 200.0 + k as f32 * 180.0,
                    y: 80.0 + (k as f32 * 37.0) % 60.0,
                    vx: a.cos() * 20.0,
                    vy: 20.0,
                    life: 1.8,
                    max: 1.8,
                    size: 10.0,
                    rot: 0.0,
                    col: GOLD,
                    _tier: tier,
                });
            }
        }
    }

    fn step_fx(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.life -= dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.rot += dt * 2.0;
            match p.kind {
                0 => {
                    p.vx += (self.time * 3.0 + p.rot).sin() * 20.0 * dt;
                    p.vy += 8.0 * dt;
                }
                1 => {
                    p.vy += 420.0 * dt;
                    if p.y > self.floor - 28.0 {
                        p.y = self.floor - 28.0;
                        p.vy *= -0.15;
                        p.life = p.life.min(0.15);
                    }
                }
                3 => {
                    p.vy += 70.0 * dt;
                    p.vx *= (-0.6 * dt).exp();
                }
                _ => {}
            }
        }
        self.particles.retain(|p| p.life > 0.0);
        if self.particles.len() > 800 {
            let n = self.particles.len() - 800;
            self.particles.drain(0..n);
        }
    }

    pub fn spawn_ambient(&mut self, dt: f32, active: bool) {
        let rate = if active { 2.0 } else { 0.6 };
        if self.rng.f32() < rate * dt {
            self.particles.push(Fx {
                kind: 4,
                x: self.rng.f32() * VW,
                y: 40.0 + self.rng.f32() * 260.0,
                vx: 8.0 + self.rng.f32() * 10.0,
                vy: 4.0,
                life: 4.0 + self.rng.f32() * 3.0,
                max: 6.0,
                size: 2.0 + self.rng.f32() * 2.0,
                rot: 0.0,
                col: rgba(255, 230, 190, 1.0),
                _tier: 0,
            });
        }
    }

    pub fn fade_ambient(&mut self) {
        for p in &mut self.particles {
            if p.kind == 4 {
                p.life = p.life.min(0.5);
            }
        }
    }

    // ------------------------------------------------------------------ drawing
    /// `props`: same camera as `cx` but faded by the transparent-mode props setting (room floor + bowl)
    pub fn draw_world(&self, cx: &Cx, props: &Cx, bg: &str, reveal_base: f32, max_visible: usize) {
        let scene = bg == "scene";
        draw_room(props, scene);
        self.draw_fx(cx, true);
        let pour = self.shows.front().and_then(|s| {
            if s.front && s.phase(reveal_base.max(1.5) + s.tier as f32 * 0.5) == "pour" {
                Some((
                    treat_for(s.tier),
                    ((s.t - s.walk_dur) / POUR).clamp(0.0, 1.0),
                ))
            } else {
                None
            }
        });
        draw_bowl(props, self.bowl_food, self.bowl_treat, pour);
        let cap = max_visible.clamp(1, 12);
        let mut vis: Vec<&CatShow> = self.shows.iter().take(cap).collect();
        vis.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for s in vis {
            let reveal = reveal_base.max(1.5) + s.tier as f32 * 0.5;
            let pose = pose_of(s, reveal, cx.time);
            let alpha = if s.phase(reveal) == "leave" {
                1.0 - smooth((s.t - s.reveal_end(reveal)) / LEAVE)
            } else {
                1.0
            };
            let head_top = cat_head_top(s.x, cx.ground(), &s.look, &pose);
            if alpha > 0.02 {
                let g = cx.ground();
                cx.halo(s.x, (head_top + g) / 2.0, 72.0 * s.look.scale, (g - head_top) / 2.0 + 12.0);
            }
            draw_cat(cx, s.x, cx.ground(), &s.look, &pose, alpha);
            draw_cat_name_tag(cx, s, head_top, alpha);
        }
        self.draw_fx(cx, false);
        draw_foreground(cx, scene);
    }

    pub fn draw_overlay(&self, cx: &Cx, thanks_title: &str, reveal_base: f32) {
        let Some(s) = self.shows.front() else { return };
        if !s.front {
            return;
        }
        let reveal = reveal_base.max(1.5) + s.tier as f32 * 0.5;
        let rt = s.t - s.walk_dur - POUR - EAT;
        if rt < 0.0 || rt > reveal {
            return;
        }
        draw_thanks_card(cx, s, rt, reveal, thanks_title);
    }

    fn draw_fx(&self, cx: &Cx, behind: bool) {
        for p in &self.particles {
            let a = (p.life / p.max.max(0.01)).clamp(0.0, 1.0);
            match p.kind {
                0 if !behind => {
                    draw_heart(cx, p.x, p.y, p.size * (0.7 + 0.3 * a), with_a(p.col, a))
                }
                1 if !behind => {
                    cx.ellipse_rot(p.x, p.y, p.size, p.size * 0.7, p.rot, with_a(p.col, a));
                }
                2 if !behind => sparkle(cx, p.x, p.y, p.size * a, with_a(p.col, a)),
                3 if !behind => {
                    sparkle(cx, p.x, p.y, p.size * (0.8 + 0.4 * a), with_a(p.col, a));
                    cx.glow_add(p.x, p.y, p.size * 4.0, p.col, 0.25 * a);
                }
                4 if behind => cx.circle(p.x, p.y, p.size, with_a(p.col, 0.35 * a)),
                _ => {}
            }
        }
    }
}

fn food_color(t: Treat, r: f32) -> Color32 {
    match t {
        Treat::Kibble => {
            if r < 0.5 {
                rgba(180, 110, 50, 1.0)
            } else {
                rgba(150, 90, 40, 1.0)
            }
        }
        Treat::Can => rgba(210, 130, 70, 1.0),
        Treat::Fish => {
            if r < 0.5 {
                rgba(230, 200, 140, 1.0)
            } else {
                rgba(240, 150, 110, 1.0)
            }
        }
        Treat::Sashimi => rgba(255, 120, 100, 1.0),
        Treat::Feast => {
            if r < 0.33 {
                rgba(255, 140, 90, 1.0)
            } else if r < 0.66 {
                GOLD
            } else {
                rgba(255, 220, 160, 1.0)
            }
        }
    }
}

pub struct CatPose {
    pub walk: f32,
    pub phase: f32,
    pub facing: f32,
    pub eat: f32,
    pub happy: f32,
    pub blink: f32,
    pub sit: f32,
    pub hop: f32,
}

fn pose_of(s: &CatShow, reveal: f32, time: f32) -> CatPose {
    let ph = s.phase(reveal);
    let blink = {
        let period = 3.0 + s.look.seed * 1.6;
        let p = (time + s.look.seed * 5.0).rem_euclid(period);
        if p < 0.12 {
            1.0 - ((p / 0.12) * 2.0 - 1.0).abs()
        } else {
            0.0
        }
    };
    match ph {
        "walk" | "leave" => {
            let facing = 1.0;
            let spd = if ph == "leave" { 11.0 } else { 10.0 };
            CatPose {
                walk: 1.0,
                phase: s.qage * spd,
                facing,
                eat: 0.0,
                happy: if ph == "leave" { 0.7 } else { 0.15 },
                blink,
                sit: 0.0,
                hop: if s.tier >= 4 && ph == "walk" {
                    (s.t * 6.0).sin().max(0.0) * 6.0
                } else {
                    0.0
                },
            }
        }
        "pour" => CatPose {
            walk: 0.0,
            phase: s.t * 4.0,
            facing: 1.0,
            eat: 0.15,
            happy: 0.4,
            blink,
            sit: 1.0,
            hop: 0.0,
        },
        "eat" | "card" => {
            let chew = 0.35 + 0.65 * (0.5 + 0.5 * (s.t * 8.0).sin());
            CatPose {
                walk: 0.0,
                phase: s.t * 6.0,
                facing: 1.0,
                eat: if ph == "eat" { chew } else { 0.1 },
                happy: 1.0,
                blink: 0.0,
                sit: 1.0,
                hop: 0.0,
            }
        }
        _ => {
            // waiting in line: loaf / sit, occasional weight shift
            let moving = (s.x - slot_x(1).min(s.x + 200.0)).abs() > 8.0
                && s.x < stand_x() - 40.0
                && (s.x - OFF_X) > 30.0;
            // if still approaching slot, walk
            // we don't have target here; use qage vs settled heuristic: walk if far from a slot-like position
            let near_slot = (s.x - s.target_x).abs() < 10.0 || (s.x - stand_x()).abs() < 10.0;
            if !near_slot && moving {
                CatPose {
                    walk: 1.0,
                    phase: s.qage * 10.0,
                    facing: 1.0,
                    eat: 0.0,
                    happy: 0.1,
                    blink,
                    sit: 0.0,
                    hop: 0.0,
                }
            } else {
                CatPose {
                    walk: 0.0,
                    phase: time + s.look.seed * 8.0,
                    facing: 1.0,
                    eat: 0.0,
                    happy: 0.2,
                    blink,
                    sit: 1.0,
                    hop: 0.0,
                }
            }
        }
    }
}

fn cat_head_top(_x: f32, g: f32, look: &CatLook, pose: &CatPose) -> f32 {
    let k = look.scale;
    let bob = if pose.walk > 0.5 {
        (pose.phase * 2.0).sin() * 6.0 * k
    } else {
        0.0
    };
    let body_y = g - 28.0 * k - pose.hop - bob;
    let head_r = 40.0 * k * look.fluff.min(1.25);
    body_y - 62.0 * k - head_r * 0.95 - if look.crown { 28.0 * k } else { 6.0 }
}

pub fn draw_cat(cx: &Cx, x: f32, g: f32, look: &CatLook, pose: &CatPose, alpha: f32) {
    if alpha <= 0.02 {
        return;
    }
    let k = look.scale;
    let f = pose.facing;
    let bob = if pose.walk > 0.5 {
        (pose.phase * 2.0).sin() * 6.0 * k
    } else {
        (cx.time * 1.6 + look.seed * 4.0).sin() * 1.4 * k
    };
    let body_y = g - 28.0 * k - pose.hop - bob * 0.6;
    let sit = pose.sit;
    let fur = with_a(look.fur, alpha);
    let dark = with_a(look.fur_dark, alpha);
    let light = with_a(look.fur_light, alpha);
    let fluff = look.fluff;

    // shadow
    cx.ellipse(x, g - 4.0, 42.0 * k, 8.0 * k, rgba(0, 0, 0, 0.22 * alpha));

    // chubby tail (behind): sit = curl to the side, walk = wave out behind
    let wag = if pose.happy > 0.5 {
        (cx.time * 8.0 + look.seed).sin() * 10.0 * k
    } else if pose.walk > 0.5 {
        (pose.phase * 1.15).sin() * 14.0 * k
    } else {
        (cx.time * 1.7 + look.seed * 3.0).sin() * 6.0 * k
    };
    let t0 = (x - 30.0 * k * f, body_y - 6.0 * k + sit * 10.0 * k);
    let n = 9;
    for i in 0..n {
        let u = i as f32 / (n - 1) as f32;
        let (dx, dy) = if sit > 0.5 {
            let dx = -(6.0 + 36.0 * u) * k * f + wag * u * u * 0.4;
            let dy = 2.0 * k - 10.0 * k * u + (u * PI).sin() * 16.0 * k;
            (dx, dy)
        } else {
            let dx = -(8.0 + 44.0 * u) * k * f + wag * u;
            let dy = -2.0 * k - 10.0 * k * u + (u * PI).sin() * 8.0 * k;
            (dx, dy)
        };
        let r = lerp(11.0, 4.2, u) * k * fluff.min(1.2);
        cx.circle(t0.0 + dx, t0.1 + dy, r + 1.4 * k, dark);
        cx.circle(t0.0 + dx, t0.1 + dy, r, fur);
        if i + 2 >= n {
            cx.circle(
                t0.0 + dx - 1.5 * k * f,
                t0.1 + dy - 1.5 * k,
                r * 0.55,
                light,
            );
        }
    }

    // back paws
    let step = if pose.walk > 0.5 {
        (pose.phase).sin()
    } else {
        0.0
    };
    let paw = |px: f32, lift: f32, spread: f32| {
        let y = g - 7.0 * k - lift.max(0.0) * 10.0 * k;
        cx.ellipse(px, y, 11.0 * k, 7.0 * k, dark);
        cx.ellipse(px + spread, y + 1.0, 6.0 * k, 5.0 * k, fur);
    };
    if sit < 0.5 {
        paw(x - 16.0 * k * f - step * 10.0 * k, step, -3.0 * k * f);
        paw(x + 18.0 * k * f + step * 10.0 * k, -step, 3.0 * k * f);
    } else {
        // tucked loaf paws
        cx.ellipse(x - 22.0 * k, g - 8.0 * k, 14.0 * k, 7.0 * k, dark);
        cx.ellipse(x + 18.0 * k, g - 8.0 * k, 14.0 * k, 7.0 * k, dark);
    }

    // body
    let bw = 36.0 * k * fluff;
    let bh = if sit > 0.5 {
        24.0 * k * fluff
    } else {
        28.0 * k * fluff
    };
    let bx = x + 2.0 * k * f;
    let by = body_y + sit * 4.0 * k;
    cx.ellipse(bx, by, bw + 2.0, bh + 2.0, dark);
    cx.ellipse(bx, by, bw, bh, fur);
    cx.ellipse(bx + 6.0 * k * f, by - 4.0 * k, bw * 0.55, bh * 0.6, light);
    // belly
    cx.ellipse(bx + 2.0 * k * f, by + 6.0 * k, bw * 0.55, bh * 0.45, light);
    if look.pattern == 2 {
        // tabby stripes
        for i in 0..3 {
            let sy = by - 10.0 * k + i as f32 * 8.0 * k;
            cx.ellipse(
                bx - 6.0 * k * f,
                sy,
                bw * 0.5,
                2.4 * k,
                with_a(look.fur_dark, 0.45 * alpha),
            );
        }
    } else if look.pattern == 4 {
        // tuxedo bib
        cx.ellipse(
            bx + 4.0 * k * f,
            by + 2.0 * k,
            bw * 0.4,
            bh * 0.55,
            with_a(Color32::from_rgb(250, 248, 242), alpha),
        );
    } else if look.pattern == 3 {
        cx.ellipse(
            bx - 14.0 * k,
            by - 6.0 * k,
            12.0 * k,
            10.0 * k,
            with_a(rgba(230, 90, 60, 1.0), 0.7 * alpha),
        );
        cx.ellipse(
            bx + 16.0 * k,
            by + 4.0 * k,
            11.0 * k,
            9.0 * k,
            with_a(Color32::from_rgb(40, 36, 36), 0.55 * alpha),
        );
    }

    // front loaf or standing chest
    if sit < 0.5 {
        paw(x + 8.0 * k * f - step * 8.0 * k, -step * 0.6, 2.0 * k * f);
        paw(x - 4.0 * k * f + step * 8.0 * k, step * 0.6, -2.0 * k * f);
    }

    // collar
    if look.collar {
        let cy = by - bh * 0.7;
        cx.ellipse(
            x,
            cy,
            20.0 * k,
            6.0 * k,
            if look.jewels {
                with_a(GOLD, alpha)
            } else {
                with_a(rgba(200, 40, 50, 1.0), alpha)
            },
        );
        let bell_x = x + 2.0 * k * f;
        let bell_y = cy + 10.0 * k + (pose.phase * 2.0).sin() * 2.0 * k * pose.walk;
        cx.circle(bell_x, bell_y, 6.0 * k, with_a(GOLD, alpha));
        cx.circle(bell_x, bell_y + 1.5 * k, 2.2 * k, with_a(GOLD_D, alpha));
        if look.jewels {
            cx.circle(
                x - 10.0 * k,
                cy,
                3.0 * k,
                with_a(rgba(120, 200, 255, 1.0), alpha),
            );
            cx.circle(
                x + 10.0 * k,
                cy,
                3.0 * k,
                with_a(rgba(255, 120, 160, 1.0), alpha),
            );
        }
    }

    // head
    let dip = pose.eat * 14.0 * k;
    let hx = x + (10.0 * (1.0 - sit) + 14.0 * pose.eat) * k * f;
    let hy = body_y - 36.0 * k * fluff.min(1.15) - (1.0 - sit) * 6.0 * k + dip;
    let hr = 36.0 * k * fluff.min(1.22);
    // ears (behind head edge)
    let ear = |side: f32, tip_dx: f32| {
        let ex = hx + side * hr * 0.72;
        let ey = hy - hr * 0.55;
        let flop = (cx.time * 1.3 + look.seed * 2.0 + side).sin() * 2.0 * k;
        let tip = (ex + side * 8.0 * k + tip_dx, ey - 22.0 * k - flop);
        let a = (ex - side * 8.0 * k, ey + 6.0 * k);
        let b = (ex + side * 14.0 * k, ey + 8.0 * k);
        cx.poly(&[a, tip, b], dark);
        cx.poly(
            &[
                (lerp(a.0, tip.0, 0.15), lerp(a.1, tip.1, 0.15)),
                (lerp(tip.0, tip.0, 1.0), lerp(tip.1, tip.1, 1.0) + 4.0 * k),
                (lerp(b.0, tip.0, 0.15), lerp(b.1, tip.1, 0.15)),
            ],
            fur,
        );
        cx.poly(
            &[
                (lerp(a.0, tip.0, 0.35), lerp(a.1, tip.1, 0.35)),
                (tip.0, tip.1 + 8.0 * k),
                (lerp(b.0, tip.0, 0.35), lerp(b.1, tip.1, 0.35)),
            ],
            with_a(rgba(255, 170, 180, 1.0), alpha),
        );
    };
    ear(-1.0, if look.pattern == 5 { -2.0 } else { 0.0 });
    ear(1.0, 0.0);
    if look.bow {
        let bx0 = hx - hr * 0.7;
        let by0 = hy - hr * 0.55;
        let bow_c = with_a(rgba(230, 60, 100, 1.0), alpha);
        cx.ellipse(bx0 - 8.0 * k, by0, 9.0 * k, 7.0 * k, bow_c);
        cx.ellipse(bx0 + 8.0 * k, by0, 9.0 * k, 7.0 * k, bow_c);
        cx.circle(bx0, by0, 4.0 * k, with_a(GOLD, alpha));
    }

    cx.circle(hx, hy, hr + 1.5 * k, dark);
    cx.circle(hx, hy, hr, fur);
    cx.ellipse(hx + 4.0 * k * f, hy - 4.0 * k, hr * 0.55, hr * 0.5, light);
    // cheeks
    cx.ellipse(
        hx - hr * 0.55,
        hy + hr * 0.15,
        10.0 * k,
        7.0 * k,
        with_a(rgba(255, 150, 160, 1.0), 0.45 * alpha),
    );
    cx.ellipse(
        hx + hr * 0.55,
        hy + hr * 0.15,
        10.0 * k,
        7.0 * k,
        with_a(rgba(255, 150, 160, 1.0), 0.45 * alpha),
    );
    if look.pattern == 2 {
        cx.seg(
            (hx - 6.0 * k, hy - hr * 0.15),
            (hx - 16.0 * k, hy - hr * 0.55),
            2.2 * k,
            dark,
        );
        cx.seg(
            (hx + 6.0 * k, hy - hr * 0.15),
            (hx + 16.0 * k, hy - hr * 0.55),
            2.2 * k,
            dark,
        );
        cx.seg((hx, hy - hr * 0.1), (hx, hy - hr * 0.7), 2.0 * k, dark);
    }
    if look.pattern == 5 {
        // siamese mask
        cx.ellipse(
            hx,
            hy + 6.0 * k,
            hr * 0.7,
            hr * 0.55,
            with_a(look.fur_dark, 0.35 * alpha),
        );
    }

    // face
    let eye_y = hy - 2.0 * k;
    let eye_dx = hr * 0.38;
    let erx = 9.5 * k;
    let ery = 11.0 * k * (1.0 - pose.blink * 0.92);
    if pose.happy > 0.55 {
        // happy ^ ^ or closed curve
        for side in [-1.0, 1.0] {
            let ex = hx + side * eye_dx;
            let w = 8.0 * k;
            cx.polyline(
                &[
                    (ex - w, eye_y + 2.0 * k),
                    (ex, eye_y - 4.0 * k),
                    (ex + w, eye_y + 2.0 * k),
                ],
                2.8 * k,
                with_a(rgba(40, 30, 30, 1.0), alpha),
            );
        }
    } else if ery > 1.5 * k {
        for side in [-1.0, 1.0] {
            let ex = hx + side * eye_dx;
            cx.ellipse(
                ex,
                eye_y,
                erx + 1.0,
                ery + 1.0,
                with_a(Color32::WHITE, alpha),
            );
            cx.ellipse(ex, eye_y, erx * 0.72, ery * 0.85, with_a(look.eye, alpha));
            cx.ellipse(
                ex,
                eye_y,
                4.0 * k,
                ery * 0.7,
                with_a(rgba(20, 24, 30, 1.0), alpha),
            );
            cx.circle(
                ex - 2.6 * k,
                eye_y - 2.8 * k,
                2.4 * k,
                with_a(Color32::WHITE, alpha),
            );
        }
    } else {
        for side in [-1.0, 1.0] {
            cx.seg(
                (hx + side * eye_dx - 7.0 * k, eye_y),
                (hx + side * eye_dx + 7.0 * k, eye_y),
                2.2 * k,
                with_a(rgba(40, 30, 30, 1.0), alpha),
            );
        }
    }
    // nose + mouth
    let nx = hx + 2.0 * k * f * 0.2;
    let ny = hy + hr * 0.22;
    cx.poly(
        &[
            (nx, ny - 2.5 * k),
            (nx + 5.0 * k, ny + 3.0 * k),
            (nx - 5.0 * k, ny + 3.0 * k),
        ],
        with_a(rgba(255, 130, 150, 1.0), alpha),
    );
    if pose.happy > 0.4 {
        cx.polyline(
            &[
                (nx - 8.0 * k, ny + 7.0 * k),
                (nx, ny + 12.0 * k),
                (nx + 8.0 * k, ny + 7.0 * k),
            ],
            2.0 * k,
            with_a(rgba(80, 40, 40, 1.0), alpha),
        );
    } else {
        cx.seg(
            (nx, ny + 3.0 * k),
            (nx, ny + 8.0 * k),
            1.6 * k,
            with_a(rgba(80, 40, 40, 1.0), alpha),
        );
        cx.polyline(
            &[
                (nx - 6.0 * k, ny + 10.0 * k),
                (nx, ny + 8.0 * k),
                (nx + 6.0 * k, ny + 10.0 * k),
            ],
            1.6 * k,
            with_a(rgba(80, 40, 40, 1.0), alpha),
        );
    }
    // whiskers
    for side in [-1.0, 1.0] {
        let wx = hx + side * hr * 0.2;
        let wy = ny + 2.0 * k;
        for j in 0..3 {
            let dy = (j as f32 - 1.0) * 4.5 * k;
            cx.seg(
                (wx, wy + dy),
                (
                    wx + side * (22.0 + j as f32) * k,
                    wy + dy - 2.0 * k + j as f32,
                ),
                1.2 * k,
                with_a(rgba(60, 50, 50, 0.75), alpha),
            );
        }
    }

    // crown
    if look.crown {
        let cy = hy - hr * 0.95;
        cx.rect(
            hx - 20.0 * k,
            cy,
            hx + 20.0 * k,
            cy + 8.0 * k,
            2.0,
            with_a(GOLD, alpha),
        );
        for (i, dx) in [-16.0, -6.0, 6.0, 16.0].iter().enumerate() {
            let hgt = if i == 1 || i == 2 { 18.0 } else { 12.0 };
            cx.poly(
                &[
                    (hx + dx * k - 5.0 * k, cy),
                    (hx + dx * k + 5.0 * k, cy),
                    (hx + dx * k, cy - hgt * k),
                ],
                with_a(GOLD, alpha),
            );
        }
        cx.circle(
            hx,
            cy - 20.0 * k,
            4.5 * k,
            with_a(rgba(255, 80, 110, 1.0), alpha),
        );
        cx.glow(hx, cy - 8.0 * k, 36.0 * k, GOLD, 0.35 * alpha);
    } else if look.jewels {
        cx.glow(hx, hy, hr * 1.6, GOLD, 0.16 * alpha);
    }
}

fn draw_heart(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    cx.circle(x - r * 0.45, y, r * 0.55, c);
    cx.circle(x + r * 0.45, y, r * 0.55, c);
    cx.poly(
        &[
            (x - r * 0.95, y + r * 0.1),
            (x + r * 0.95, y + r * 0.1),
            (x, y + r * 1.25),
        ],
        c,
    );
}

pub fn draw_room(cx: &Cx, opaque: bool) {
    let g = cx.ground();
    let t = cx.time;
    if opaque {
        cx.vgradient(
            -40.0,
            -40.0,
            VW + 40.0,
            g,
            rgba(255, 226, 200, 1.0),
            rgba(255, 210, 180, 1.0),
        );
        // wallpaper dots
        let mut y = 20.0;
        let mut row = 0;
        while y < g - 20.0 {
            let mut x = if row % 2 == 0 { 16.0 } else { 40.0 };
            while x < VW {
                cx.circle(x, y, 3.0, rgba(255, 180, 150, 0.25));
                x += 48.0;
            }
            y += 42.0;
            row += 1;
        }
        // window
        let (wx, wy, ww, wh) = (160.0, 40.0, 260.0, 210.0);
        cx.rect(
            wx - 12.0,
            wy - 12.0,
            wx + ww + 12.0,
            wy + wh + 12.0,
            8.0,
            rgba(210, 150, 110, 1.0),
        );
        cx.vgradient(
            wx,
            wy,
            wx + ww,
            wy + wh,
            rgba(160, 210, 255, 1.0),
            rgba(255, 230, 200, 1.0),
        );
        // soft sun
        cx.glow(
            wx + ww * 0.7,
            wy + 40.0,
            80.0,
            rgba(255, 240, 180, 1.0),
            0.45,
        );
        cx.rect(
            wx + ww / 2.0 - 4.0,
            wy,
            wx + ww / 2.0 + 4.0,
            wy + wh,
            0.0,
            rgba(230, 180, 130, 0.65),
        );
        cx.rect(
            wx,
            wy + wh / 2.0 - 4.0,
            wx + ww,
            wy + wh / 2.0 + 4.0,
            0.0,
            rgba(230, 180, 130, 0.65),
        );
        // curtains
        cx.vgradient(
            wx - 8.0,
            wy - 6.0,
            wx + 36.0,
            wy + wh + 8.0,
            rgba(255, 140, 150, 0.95),
            rgba(230, 90, 110, 0.95),
        );
        cx.vgradient(
            wx + ww - 36.0,
            wy - 6.0,
            wx + ww + 8.0,
            wy + wh + 8.0,
            rgba(255, 140, 150, 0.95),
            rgba(230, 90, 110, 0.95),
        );
        // sill plant
        cx.rect(
            wx - 4.0,
            wy + wh,
            wx + ww + 4.0,
            wy + wh + 16.0,
            4.0,
            rgba(190, 130, 90, 1.0),
        );
        let px = wx + ww - 40.0;
        cx.rect(
            px - 10.0,
            wy + wh - 2.0,
            px + 10.0,
            wy + wh + 14.0,
            3.0,
            rgba(200, 90, 70, 1.0),
        );
        for i in 0..5 {
            let a = -1.4 + i as f32 * 0.55;
            cx.ellipse(
                px + a.cos() * 8.0,
                wy + wh - 18.0 - i as f32 * 6.0,
                10.0,
                16.0,
                rgba(80, 160, 90, 1.0),
            );
        }
        // shelf
        cx.rect(820.0, 90.0, 1040.0, 102.0, 3.0, rgba(190, 130, 90, 1.0));
        cx.rect(860.0, 62.0, 890.0, 90.0, 4.0, rgba(230, 150, 80, 1.0));
        cx.rect(900.0, 50.0, 934.0, 90.0, 4.0, rgba(90, 160, 200, 1.0));
        cx.circle(980.0, 78.0, 16.0, rgba(255, 200, 80, 1.0));
        // string lights
        let mut prev = (400.0, 16.0);
        for i in 0..10 {
            let u = i as f32 / 9.0;
            let p = (400.0 + u * 640.0, 18.0 + (u * PI).sin() * 18.0);
            cx.seg(prev, p, 1.4, rgba(120, 80, 60, 0.7));
            prev = p;
            if i > 0 && i < 9 {
                let fl = 0.75 + 0.25 * (t * 2.2 + i as f32).sin();
                let c = [
                    rgba(255, 200, 90, 1.0),
                    rgba(255, 140, 150, 1.0),
                    rgba(180, 220, 255, 1.0),
                ][i % 3];
                cx.glow(p.0, p.1 + 10.0, 18.0, c, 0.4 * fl);
                cx.ellipse(p.0, p.1 + 10.0, 5.0, 7.0, c);
            }
        }
        // cat tree
        cx.rect(70.0, g - 200.0, 92.0, g, 4.0, rgba(200, 160, 120, 1.0));
        cx.ellipse(81.0, g - 200.0, 36.0, 10.0, rgba(210, 140, 90, 1.0));
        cx.ellipse(110.0, g - 130.0, 32.0, 9.0, rgba(210, 140, 90, 1.0));
        // floor
        cx.vgradient(
            -40.0,
            g,
            VW + 40.0,
            cx.vh + 40.0,
            rgba(210, 150, 110, 1.0),
            rgba(160, 100, 70, 1.0),
        );
        let mut fy = g + 18.0;
        while fy < cx.vh + 20.0 {
            cx.seg((-40.0, fy), (VW + 40.0, fy), 2.0, rgba(150, 90, 60, 0.35));
            fy += 22.0;
        }
    } else {
        // overlay: just a cozy rug + floor hint
        cx.ellipse(VW * 0.55, g + 8.0, 420.0, 28.0, rgba(200, 90, 90, 0.35));
        cx.vgradient(
            -20.0,
            g + 2.0,
            VW + 20.0,
            g + 70.0,
            rgba(190, 140, 100, 0.5),
            rgba(190, 140, 100, 0.0),
        );
    }
    // cushions + yarn always (sit on the floor)
    cx.ellipse(230.0, g - 10.0, 54.0, 16.0, rgba(255, 160, 170, 0.95));
    cx.ellipse(230.0, g - 22.0, 48.0, 18.0, rgba(255, 190, 195, 1.0));
    cx.ellipse(320.0, g - 8.0, 40.0, 13.0, rgba(160, 200, 230, 0.95));
    cx.ellipse(320.0, g - 18.0, 36.0, 14.0, rgba(190, 220, 240, 1.0));
    // yarn
    let yx = 400.0;
    cx.circle(yx, g - 12.0, 14.0, rgba(255, 110, 130, 1.0));
    cx.circle(yx - 3.0, g - 14.0, 10.0, rgba(255, 150, 160, 1.0));
    cx.polyline(
        &[
            (yx + 12.0, g - 10.0),
            (yx + 34.0, g - 2.0),
            (yx + 58.0, g - 8.0),
        ],
        2.0,
        rgba(255, 110, 130, 1.0),
    );
}

pub fn draw_bowl(cx: &Cx, food: f32, treat: Treat, pour: Option<(Treat, f32)>) {
    let g = cx.ground();
    let x = BOWL_X;
    // mat
    cx.ellipse(x, g - 4.0, 86.0, 18.0, rgba(120, 180, 150, 0.9));
    cx.ellipse(x, g - 4.0, 70.0, 13.0, rgba(150, 210, 180, 0.9));
    // bowl
    let rim_y = g - 28.0;
    cx.ellipse(x, g - 10.0, 48.0, 12.0, rgba(90, 70, 70, 1.0));
    cx.ellipse(x, rim_y + 10.0, 52.0, 16.0, rgba(250, 245, 240, 1.0));
    cx.ellipse(x, rim_y + 10.0, 44.0, 12.0, rgba(255, 236, 220, 1.0));
    cx.ellipse(x, rim_y, 54.0, 14.0, rgba(255, 250, 245, 1.0));
    cx.ring(x, rim_y, 54.0, 4.0, GOLD_D);
    cx.ring(x, rim_y, 48.0, 2.5, rgba(255, 220, 160, 1.0));
    let fill = food.clamp(0.0, 1.0);
    if fill > 0.02 {
        let fy = rim_y + 6.0 + (1.0 - fill) * 8.0;
        let col = match treat {
            Treat::Kibble => rgba(180, 110, 50, 1.0),
            Treat::Can => rgba(200, 120, 70, 1.0),
            Treat::Fish => rgba(240, 200, 130, 1.0),
            Treat::Sashimi => rgba(255, 130, 110, 1.0),
            Treat::Feast => rgba(255, 170, 90, 1.0),
        };
        cx.ellipse(x, fy, 38.0 * (0.7 + 0.3 * fill), 9.0 * fill + 4.0, col);
        if matches!(treat, Treat::Fish | Treat::Feast | Treat::Sashimi) {
            cx.ellipse(x + 6.0, fy - 3.0, 16.0, 6.0, rgba(255, 220, 160, 1.0));
            cx.ellipse(x - 10.0, fy - 1.0, 10.0, 4.0, rgba(255, 150, 110, 1.0));
        }
        if treat == Treat::Feast {
            cx.glow(x, fy - 10.0, 70.0, GOLD, 0.35);
        }
    }
    if let Some((tr, u)) = pour {
        let y0 = rim_y - 110.0;
        let y1 = rim_y - 8.0;
        let y = lerp(y0, y1, ease_out_cubic(u));
        // stream
        cx.rect(
            x - 3.0,
            y0,
            x + 3.0,
            y,
            2.0,
            with_a(food_color(tr, 0.3), 0.8),
        );
        if matches!(tr, Treat::Fish | Treat::Feast) && u > 0.35 {
            // a little fish dropping in
            let fy = lerp(y0, y1, ease_in_out(u));
            draw_mini_fish(cx, x + 16.0, fy, 0.9, 0.3);
        }
        if tr == Treat::Feast {
            cx.glow(x, y0 + 20.0, 50.0, GOLD, 0.3);
        }
    }
}

fn draw_mini_fish(cx: &Cx, x: f32, y: f32, sc: f32, ang: f32) {
    let (s, c) = ang.sin_cos();
    let p = |dx: f32, dy: f32| (x + dx * sc * c - dy * sc * s, y + dx * sc * s + dy * sc * c);
    cx.poly(
        &[p(-16.0, 0.0), p(10.0, -8.0), p(16.0, 0.0), p(10.0, 8.0)],
        rgba(255, 170, 90, 1.0),
    );
    cx.poly(
        &[p(-16.0, 0.0), p(-26.0, -8.0), p(-22.0, 0.0), p(-26.0, 8.0)],
        rgba(255, 130, 80, 1.0),
    );
    cx.circle(
        p(8.0, -1.0).0,
        p(8.0, -1.0).1,
        2.0 * sc,
        rgba(40, 30, 30, 1.0),
    );
}

pub fn draw_foreground(cx: &Cx, _scene: bool) {
    let g = cx.ground();
    // a little plant in the front-right corner
    cx.ellipse(1020.0, g - 4.0, 36.0, 10.0, rgba(80, 120, 70, 0.5));
    for i in 0..6 {
        let a = -1.6 + i as f32 * 0.45;
        cx.ellipse(
            1020.0 + a.cos() * 16.0,
            g - 22.0 - i as f32 * 7.0,
            12.0,
            18.0,
            rgba(70, 150, 80, 0.9),
        );
    }
}

fn draw_cat_name_tag(cx: &Cx, s: &CatShow, head_top: f32, alpha: f32) {
    let appear = s.qage / 0.85;
    if s.front {
        let name = ellipsize_name(&s.user, 14);
        let ribbon = if s.tier >= 2 {
            s.tier_name.as_str()
        } else {
            ""
        };
        draw_name_banner(cx, s.x, head_top, &name, s.tier, ribbon, appear, alpha);
    } else {
        draw_compact_name_tag(
            cx,
            s.x,
            head_top,
            &ellipsize_name(&s.user, 8),
            s.tier,
            appear,
            alpha,
        );
    }
}

fn draw_compact_name_tag(
    cx: &Cx,
    x: f32,
    y: f32,
    name: &str,
    tier: usize,
    appear: f32,
    alpha: f32,
) {
    let sc = ease_out_back(appear).max(0.0);
    if sc <= 0.02 || alpha <= 0.0 {
        return;
    }
    let size = 24.0 * sc;
    let m = cx.measure(name, size, true);
    let w = m.x + 22.0 * sc;
    let h = size * 1.45;
    let (x0, y0) = (x - w / 2.0, y - h);
    let a = alpha;
    let fill = if tier >= 2 {
        rgba(20, 20, 40, 0.82 * a)
    } else {
        rgba(0, 0, 0, 0.62 * a)
    };
    cx.rect(x0, y0, x0 + w, y, h / 2.0, fill);
    if tier >= 2 {
        cx.rect_line(x0, y0, x0 + w, y, h / 2.0, 2.0 * sc, with_a(GOLD, a));
        cx.text(
            x,
            y - h / 2.0,
            Align2::CENTER_CENTER,
            name,
            size,
            with_a(GOLD, a),
            true,
        );
    } else {
        cx.text(
            x,
            y - h / 2.0,
            Align2::CENTER_CENTER,
            name,
            size,
            with_a(Color32::WHITE, a),
            true,
        );
    }
}

fn card_frame(cx: &Cx, cxx: f32, top: f32, wdt: f32, hgt: f32, alpha: f32, tier: usize) {
    let (x0, x1) = (cxx - wdt / 2.0, cxx + wdt / 2.0);
    if tier >= 3 {
        cx.glow(cxx, top + hgt / 2.0, wdt * 0.75, GOLD, 0.35 * alpha);
    }
    cx.rect(
        x0 + 8.0,
        top + 10.0,
        x1 + 8.0,
        top + hgt + 10.0,
        24.0,
        rgba(0, 0, 0, 0.25 * alpha),
    );
    cx.vgradient(
        x0,
        top,
        x1,
        top + hgt,
        rgba(255, 236, 214, 0.97 * alpha),
        rgba(255, 210, 180, 0.97 * alpha),
    );
    cx.rect_line(
        x0,
        top,
        x1,
        top + hgt,
        22.0,
        4.0 + tier as f32,
        with_a(
            if tier >= 2 {
                GOLD
            } else {
                rgba(210, 120, 80, 1.0)
            },
            alpha,
        ),
    );
    cx.rect_line(
        x0 + 12.0,
        top + 12.0,
        x1 - 12.0,
        top + hgt - 12.0,
        16.0,
        2.0,
        with_a(rgba(255, 200, 140, 1.0), 0.7 * alpha),
    );
    // paw corners
    for (px, py) in [
        (x0 + 22.0, top + 22.0),
        (x1 - 22.0, top + 22.0),
        (x0 + 22.0, top + hgt - 22.0),
        (x1 - 22.0, top + hgt - 22.0),
    ] {
        cx.circle(px, py, 6.0, with_a(rgba(230, 120, 110, 1.0), alpha));
        for (dx, dy) in [(-8.0, -6.0), (8.0, -6.0), (-9.0, 5.0), (9.0, 5.0)] {
            cx.circle(
                px + dx,
                py + dy,
                3.2,
                with_a(rgba(230, 120, 110, 1.0), alpha),
            );
        }
    }
}

fn tab_size(tier: usize) -> f32 {
    24.0 + tier as f32 * 2.0
}
fn tab_height(tier: usize) -> f32 {
    tab_size(tier) * 1.45
}
/// How far the name tab hangs above the card's top edge.
fn tab_hang(tier: usize) -> f32 {
    (tab_height(tier) - 8.0).max(8.0)
}

fn draw_name_tab(cx: &Cx, x0: f32, top: f32, name: &str, tier: usize, alpha: f32) {
    let label = format!("ของ {name}");
    let size = tab_size(tier);
    let m = cx.measure(&label, size, true);
    let tw = m.x + 28.0;
    let th = tab_height(tier);
    let hang = tab_hang(tier);
    let tx0 = x0 + 24.0;
    let ty0 = top - hang;
    cx.rect(
        tx0,
        ty0,
        tx0 + tw,
        ty0 + th,
        12.0,
        rgba(200, 70, 90, 0.95 * alpha),
    );
    cx.rect_line(tx0, ty0, tx0 + tw, ty0 + th, 12.0, 2.0, with_a(GOLD, alpha));
    cx.text_o(
        tx0 + 14.0,
        ty0 + th / 2.0,
        Align2::LEFT_CENTER,
        &label,
        size,
        with_a(Color32::WHITE, alpha),
        rgba(80, 0, 20, 0.85 * alpha),
        true,
    );
}

/// Thank-you card only (no lucky numbers). Kept fully inside the frame with top padding.
fn draw_thanks_card(cx: &Cx, s: &CatShow, rt: f32, r: f32, thanks_title: &str) {
    let tier = s.tier;
    let fo = smooth((rt - (r - 0.45)) / 0.45);
    let alpha = smooth(rt / 0.25) * (1.0 - fo);
    let hgt = 340.0;
    let wdt = 660.0 + tier as f32 * 12.0;
    let cxx = 540.0;
    let pad = 20.0;
    let settled = pad + tab_hang(tier);
    let enter = (1.0 - ease_out_back(rt / 0.55)) * (hgt + 50.0);
    let top = settled + enter - fo * 16.0;
    card_frame(cx, cxx, top, wdt, hgt, alpha, tier);
    draw_name_tab(cx, cxx - wdt / 2.0, top, &s.user, tier, alpha);

    let ca = alpha * smooth((rt - 0.2) / 0.3);
    // generous vertical rhythm so nothing overlaps; whole card sits below the hanging name tab
    if !thanks_title.trim().is_empty() {
        cx.text(
            cxx,
            top + 36.0,
            Align2::CENTER_CENTER,
            thanks_title,
            20.0,
            rgba(160, 80, 60, ca),
            false,
        );
    }
    let title = format!("ขอบคุณ {}", s.user);
    cx.text_o(
        cxx,
        top + 74.0,
        Align2::CENTER_CENTER,
        &title,
        40.0,
        with_a(rgba(150, 50, 50, 1.0), ca),
        with_a(Color32::WHITE, ca),
        true,
    );
    // hearts beside the name, fancier with tier
    let nhearts = [1, 1, 2, 3, 4][tier.min(4)];
    for i in 0..nhearts {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let k = (i / 2) as f32;
        let hx = cxx + side * (230.0 + k * 16.0);
        let hy = top + 74.0 + (k - 0.3) * 6.0;
        let pulse = 0.85 + 0.15 * (cx.time * 3.0 + i as f32).sin();
        draw_heart(
            cx,
            hx,
            hy,
            11.0 * pulse,
            with_a(rgba(255, 110, 150, 1.0), ca),
        );
    }
    cx.seg(
        (cxx - wdt * 0.32, top + 110.0),
        (cxx + wdt * 0.32, top + 110.0),
        2.0,
        with_a(GOLD, ca * 0.8),
    );
    cx.text_o(
        cxx,
        top + 142.0,
        Align2::CENTER_CENTER,
        "ที่เลี้ยงข้าวน้องแมว",
        34.0,
        with_a(rgba(120, 50, 40, 1.0), ca),
        with_a(Color32::WHITE, ca * 0.5),
        true,
    );

    let ta = alpha * smooth((rt - 0.55) / 0.4);
    if !s.text.trim().is_empty() {
        cx.text_wrapped(
            cxx,
            top + 182.0 + (1.0 - smooth((rt - 0.55) / 0.4)) * 10.0,
            &s.text,
            28.0,
            wdt - 140.0,
            rgba(90, 40, 30, ta),
            true,
        );
    }
    cx.text(
        cxx,
        top + hgt - 42.0,
        Align2::CENTER_CENTER,
        treat_for(tier).label(),
        22.0,
        rgba(160, 80, 50, ta),
        false,
    );

    // corner / border sparkles — more at higher tiers
    let ns = [4, 6, 8, 10, 14][tier.min(4)];
    for i in 0..ns {
        let a = i as f32 / ns as f32 * TAU + cx.time * 0.5;
        let rx = wdt * 0.48;
        let ry = hgt * 0.42;
        sparkle(
            cx,
            cxx + a.cos() * rx,
            top + hgt / 2.0 + a.sin() * ry,
            8.0 * (cx.time * 3.0 + i as f32).sin().abs(),
            with_a(
                if tier >= 3 {
                    GOLD
                } else {
                    rgba(255, 160, 180, 1.0)
                },
                ca,
            ),
        );
    }
    if tier >= 4 {
        cx.glow(cxx, top + 80.0, 180.0, GOLD, 0.2 * ca);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_mapping_treat_and_look() {
        assert_eq!(treat_for(0), Treat::Kibble);
        assert_eq!(treat_for(1), Treat::Can);
        assert_eq!(treat_for(2), Treat::Fish);
        assert_eq!(treat_for(3), Treat::Sashimi);
        assert_eq!(treat_for(4), Treat::Feast);
        assert_eq!(treat_for(99), Treat::Feast);
        let t1 = look_for("มะลิ", 0);
        let t5 = look_for("มะลิ", 4);
        assert!(!t1.crown && !t1.collar && !t1.bow && !t1.jewels);
        assert!(t5.crown && t5.collar && t5.bow && t5.jewels);
        assert!(t5.fluff > t1.fluff);
        assert!(t5.scale > t1.scale);
        let mid = look_for("สมชาย", 1);
        assert!(mid.collar && !mid.crown);
        // same name -> stable palette
        assert_eq!(look_for("Nana", 2).fur, look_for("Nana", 4).fur);
    }

    #[test]
    fn queue_of_three_senders() {
        let mut st = Stage::with_seed(42);
        for (name, tier) in [("มะลิ", 0usize), ("สมชาย", 2), ("เศรษฐีต้น", 4)]
        {
            st.enqueue(CatIn {
                action: Action::Pray,
                user: name.into(),
                tier,
                tier_name: format!("ระดับ{}", tier + 1),
                text: "ขอบคุณนะ".into(),
            });
        }
        assert_eq!(st.len(), 3);
        assert_eq!(st.front_user(), Some("มะลิ"));
        let reveal = 5.0;
        // ~3 seconds: all three should be on stage, front walking or pouring
        for _ in 0..180 {
            st.tick(1.0 / 60.0, 5, reveal, 540.0);
        }
        let vis = st.visible(5);
        assert_eq!(
            vis.len(),
            3,
            "expected 3 visible cats after 3s, got {} xs={:?}",
            vis.len(),
            vis.iter()
                .map(|s| (s.user.as_str(), s.x))
                .collect::<Vec<_>>()
        );
        assert_eq!(vis[0].user, "มะลิ");
        assert_eq!(vis[1].user, "สมชาย");
        assert_eq!(vis[2].user, "เศรษฐีต้น");
        assert!(vis[0].front);
        assert!(!vis[1].front && !vis[2].front);
        assert!(
            vis[0].x > vis[1].x && vis[1].x > vis[2].x,
            "line should stand left of the bowl in order"
        );
        assert!(
            vis[0].x > 500.0,
            "front cat should be near the bowl, x={}",
            vis[0].x
        );
        assert!(
            vis[2].x > 0.0,
            "third cat should have walked on screen, x={}",
            vis[2].x
        );
        // names are stored on each show
        for s in &vis {
            assert!(!s.user.is_empty());
        }
        // advance until the first cat finishes and leaves
        let mut left = false;
        for _ in 0..2000 {
            st.tick(1.0 / 60.0, 5, reveal, 540.0);
            if st.front_user() == Some("สมชาย") {
                left = true;
                break;
            }
        }
        assert!(
            left,
            "front should advance to สมชาย, now={:?}",
            st.front_user()
        );
        assert_eq!(st.len(), 2);
        assert_eq!(st.waiting_names(), vec!["เศรษฐีต้น"]);
        // second also finishes
        for _ in 0..2000 {
            st.tick(1.0 / 60.0, 5, reveal, 540.0);
            if st.front_user() == Some("เศรษฐีต้น") {
                break;
            }
        }
        assert_eq!(st.front_user(), Some("เศรษฐีต้น"));
        assert_eq!(st.len(), 1);
    }

    #[test]
    fn thanks_only_no_lucky_numbers() {
        // After eating, the overlay always draws the thank-you card, for either gift action.
        let src = include_str!("cat.rs");
        let needle = format!("fn draw_{}", "number_card");
        assert!(!src.contains(&needle), "lucky-number card must be removed");
        assert!(src.contains("fn draw_thanks_card"));
        assert!(
            !src.contains("draw_overlay") || src.contains("draw_thanks_card(cx, s, rt, reveal"),
            "overlay must dispatch only the thank-you card"
        );
        let mut st = Stage::with_seed(7);
        st.enqueue(CatIn {
            action: Action::Pray,
            user: "มะลิ".into(),
            tier: 4,
            tier_name: "จักรพรรดิเพชร".into(),
            text: "ขอบคุณนะ".into(),
        });
        let front = st.shows().next().expect("queued");
        assert!(!front.text.contains("54"));
        assert_eq!(front.user, "มะลิ");
    }

    #[test]
    fn queue_tags_do_not_overlap() {
        let names = ["มะลิ", "สมชาย", "เศรษฐีต้น"];
        let tiers = [0usize, 2, 4];
        let xs = queue_targets(&names, &tiers, 5);
        assert_eq!(xs.len(), 3);
        assert!((xs[0] - stand_x()).abs() < 0.1);
        // Adjacent tag boxes must have a gap.
        for i in 1..3 {
            let prev_half = tag_half_width(names[i - 1], i - 1 == 0, tiers[i - 1]);
            let this_half = tag_half_width(names[i], false, tiers[i]);
            let gap = (xs[i - 1] - prev_half) - (xs[i] + this_half);
            assert!(
                gap >= 19.0,
                "tags overlap or nearly touch between {} and {}: gap={gap} xs={xs:?}",
                names[i - 1],
                names[i]
            );
        }
        // Long names shrink by visible units; Thai tones don't force an ellipsis.
        assert_eq!(visible_units(&ellipsize_name("ชื่อที่ยาวมากกกกกกกก", 8)), 8);
        assert!(ellipsize_name("ชื่อที่ยาวมากกกกกกกก", 8).ends_with('…'));
        assert_eq!(ellipsize_name("มะลิ", 8), "มะลิ");
        assert_eq!(ellipsize_name("เศรษฐีต้น", 8), "เศรษฐีต้น");
        // Waiting tags are narrower than a front tag with a ribbon.
        assert!(tag_half_width("เศรษฐีต้น", false, 4) < tag_half_width("เศรษฐีต้น", true, 4));
    }
}
