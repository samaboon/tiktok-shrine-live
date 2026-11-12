//! โหมดให้อาหารแมว: มุมอบอุ่น แมวเดินเข้าคิว กินตามระดับของขวัญ แล้วขึ้นการ์ดขอบคุณ (ไม่มีเลขมงคล)
use crate::events::Action;
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

const SILVER: Color32 = Color32::from_rgb(210, 222, 232);
const SILVER_D: Color32 = Color32::from_rgb(140, 155, 170);
const CRIMSON: Color32 = Color32::from_rgb(150, 16, 36);

pub const BOWL_X: f32 = 760.0;
pub const STAND_OFF: f32 = 78.0;
pub const SLOT_GAP: f32 = 148.0;
pub const OFF_X: f32 = -180.0;
pub const POUR: f32 = 1.15;
pub const EAT: f32 = 2.25;
pub const LEAVE: f32 = 1.65;
pub const WALK_MAX: f32 = 2.4;

/// Pour / eat / leave / reveal lengths grow with gift tier so luxury stages have time to land.
pub fn pour_dur(tier: usize) -> f32 {
    POUR + [0.0, 0.06, 0.12, 0.25, 0.4][tier.min(4)]
}
pub fn eat_dur(tier: usize) -> f32 {
    EAT + [0.0, 0.2, 0.5, 1.0, 1.5][tier.min(4)]
}
pub fn leave_dur(_tier: usize) -> f32 {
    LEAVE
}
pub fn reveal_dur(tier: usize, reveal_base: f32) -> f32 {
    reveal_base.max(1.5) + [0.4, 0.6, 1.0, 1.7, 2.5][tier.min(4)]
}
pub fn walk_max(tier: usize) -> f32 {
    if tier >= 4 {
        3.2
    } else if tier >= 3 {
        2.8
    } else {
        WALK_MAX
    }
}

pub fn floor_y(vh: f32) -> f32 {
    (vh - 96.0_f32.min(vh * 0.14)).max(vh * 0.62)
}
pub fn stand_x() -> f32 {
    BOWL_X - STAND_OFF
}
/// Body scale of the front/hero cat. Tier 4 is 1.5× tier 1, tier 5 is 2×.
pub fn hero_body_scale(tier: usize) -> f32 {
    0.88 * [1.0, 1.12, 1.25, 1.50, 2.00][tier.min(4)]
}
/// Waiting/queue cats stay compact so the line stays readable.
pub fn queue_body_scale(tier: usize) -> f32 {
    [0.88, 0.92, 0.96, 1.00, 1.04][tier.min(4)]
}
/// Throne, treat tower, and attendants scale with the hero.
pub fn ceremony_scale(tier: usize) -> f32 {
    [1.0, 1.05, 1.15, 1.50, 2.00][tier.min(4)]
}
pub fn attendant_scale(tier: usize) -> f32 {
    [0.85, 0.92, 1.08, 1.38, 1.58][tier.min(4)]
}
pub fn look_drawn(s: &CatShow) -> CatLook {
    let mut look = s.look.clone();
    look.scale = if s.front {
        hero_body_scale(s.tier)
    } else {
        queue_body_scale(s.tier)
    };
    look
}

/// Front cat is centered at high tiers so the hero + throne + tower dominate the frame.
pub fn front_stand(tier: usize) -> f32 {
    match tier.min(4) {
        4 => 450.0,
        3 => 500.0,
        _ => stand_x(),
    }
}
pub fn treat_x(tier: usize) -> f32 {
    match tier.min(4) {
        4 => 790.0,
        3 => 760.0,
        _ => BOWL_X,
    }
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
    xs[0] = front_stand(tiers.first().copied().unwrap_or(0));
    for i in 1..n {
        if i >= cap {
            xs[i] = OFF_X;
            continue;
        }
        let prev_tier = tiers.get(i - 1).copied().unwrap_or(0);
        let prev_half = tag_half_width(names[i - 1], i - 1 == 0, prev_tier);
        let this_half = tag_half_width(names[i], false, tiers.get(i).copied().unwrap_or(0));
        // A high-tier front tag sits high above the throne, so don't reserve its
        // full typographic width — leave room for a readable waiter line instead.
        let (prev_for_gap, extra) = if i == 1 && prev_tier >= 3 {
            (40.0, 70.0 + 40.0 * ceremony_scale(prev_tier).sqrt())
        } else {
            (prev_half, 0.0)
        };
        let gap = (prev_for_gap + this_half + 20.0 + extra).max(130.0);
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
    pub cape: bool,
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
        fluff: [0.85, 0.96, 1.10, 1.48, 1.74][t],
        scale: [0.88, 0.94, 1.02, 1.16, 1.28][t],
        seed: ((h >> 11) % 10000) as f32 / 10000.0,
        collar: t >= 1,
        bow: t >= 2,
        jewels: t >= 3,
        crown: t >= 4,
        cape: t >= 4,
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
    pub fn pour_d(&self) -> f32 {
        pour_dur(self.tier)
    }
    pub fn eat_d(&self) -> f32 {
        eat_dur(self.tier)
    }
    pub fn leave_d(&self) -> f32 {
        leave_dur(self.tier)
    }
    pub fn reveal_end(&self, reveal: f32) -> f32 {
        self.walk_dur + self.pour_d() + self.eat_d() + reveal
    }
    pub fn total(&self, reveal: f32) -> f32 {
        self.reveal_end(reveal) + self.leave_d()
    }
    pub fn phase(&self, reveal: f32) -> &'static str {
        let t = self.t;
        let w = self.walk_dur;
        let p = self.pour_d();
        let e = self.eat_d();
        if !self.front {
            "wait"
        } else if t < w {
            "walk"
        } else if t < w + p {
            "pour"
        } else if t < w + p + e {
            "eat"
        } else if t < w + p + e + reveal {
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
    kind: u8, // 0 heart, 1 food, 2 sparkle, 3 burst, 4 mote, 5 petal, 6 confetti, 7 firework
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
            let reveal = reveal_dur(self.shows[i].tier, reveal_base);
            if i == 0 {
                if !self.shows[i].front {
                    self.shows[i].front = true;
                    self.shows[i].from_x = self.shows[i].x;
                    let dist = (front_stand(self.shows[i].tier) - self.shows[i].x).abs();
                    let speed = if self.shows[i].tier >= 4 {
                        230.0
                    } else {
                        290.0
                    };
                    self.shows[i].walk_dur =
                        (dist / speed).clamp(0.55, walk_max(self.shows[i].tier));
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
        let p = self.shows[i].pour_d();
        let e = self.shows[i].eat_d();
        let l = self.shows[i].leave_d();
        let tier = self.shows[i].tier;
        self.shows[i].t += dt;
        let t = self.shows[i].t;
        let stand = front_stand(tier);
        if t < w {
            let u = ease_in_out(t / w);
            self.shows[i].x = lerp(self.shows[i].from_x, stand, u);
        } else if t < w + p + e + reveal {
            self.shows[i].x = stand;
        } else {
            let u = ease_in_out(((t - w - p - e - reveal) / l).clamp(0.0, 1.0));
            self.shows[i].x = lerp(stand, VW + 200.0, u);
        }
        // pour food
        if t >= w && t < w + p {
            if !self.shows[i].poured {
                self.shows[i].poured = true;
                self.bowl_treat = treat_for(tier);
            }
            let fill = 0.2 + 0.75 * smooth((t - w) / p);
            self.bowl_food = self.bowl_food.max(fill);
            self.spawn_food(dt, tier);
        }
        if t >= w + p && t < w + p + e {
            self.spawn_hearts(i, dt);
            self.bowl_food = (self.bowl_food - dt * 0.12).max(0.18);
        }
        if t >= w + p + e && self.shows[i].burst_at < 0.0 {
            self.shows[i].burst_at = t;
            self.spawn_celebrate(tier);
        }
        // ongoing luxury weather
        if t < w + p + e + reveal {
            self.spawn_luxury_ambience(i, dt);
        }
        if t >= w + p && t < w + p + e + reveal && tier >= 3 {
            self.spawn_luxury_finale(tier, dt);
        }
        if t >= self.shows[i].total(reveal) {
            self.shows[i].done = true;
        }
    }

    fn spawn_food(&mut self, dt: f32, tier: usize) {
        let rate = 10.0 + tier as f32 * 10.0;
        if self.rng.f32() < rate * dt {
            let treat = treat_for(tier);
            let col = food_color(treat, self.rng.f32());
            let x = treat_x(tier) + (self.rng.f32() - 0.5) * 36.0;
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
        let cat = &self.shows[i];
        let tier = cat.tier;
        let x = cat.x;
        let sc = if cat.front {
            hero_body_scale(tier)
        } else {
            queue_body_scale(tier)
        };
        // Hearts and sparkles stay above or beside the face so the muzzle stays visible.
        let face_r = 52.0 * sc;
        let crown = if cat.look.crown { 36.0 * sc } else { 0.0 };
        let head_top = self.floor - 28.0 * sc - 72.0 * sc - crown;
        let rate = 3.5 + tier as f32 * 3.2;
        if self.rng.f32() < rate * dt {
            let side = if self.rng.bool() { -1.0 } else { 1.0 };
            let around = self.rng.bool();
            let (px, py) = if around {
                (
                    x + side * (face_r + 16.0 + self.rng.f32() * 36.0),
                    head_top + 8.0 + self.rng.f32() * 32.0 * sc,
                )
            } else {
                (
                    x + (self.rng.f32() - 0.5) * face_r * 1.35,
                    head_top - 20.0 - self.rng.f32() * 48.0,
                )
            };
            self.particles.push(Fx {
                kind: 0,
                x: px,
                y: py,
                vx: side * (16.0 + self.rng.f32() * 22.0),
                vy: -(50.0 + self.rng.f32() * 36.0),
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
            let side = if self.rng.bool() { -1.0 } else { 1.0 };
            self.particles.push(Fx {
                kind: 2,
                x: x + side * (face_r + 12.0 + self.rng.f32() * 40.0),
                y: head_top - 10.0 - self.rng.f32() * 50.0,
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
        let n = [8, 16, 28, 64, 120][tier.min(4)];
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
                x: treat_x(tier),
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
        if tier >= 3 {
            let bursts = if tier >= 4 { 12 } else { 6 };
            let fw_cols = [
                GOLD,
                rgba(220, 24, 40, 1.0),
                rgba(255, 140, 30, 1.0),
                rgba(255, 60, 90, 1.0),
                rgba(255, 210, 50, 1.0),
                rgba(180, 20, 50, 1.0),
            ];
            for k in 0..bursts {
                let a = k as f32 / bursts as f32 * TAU + 0.4;
                self.particles.push(Fx {
                    kind: 7,
                    x: 80.0 + k as f32 * (VW - 160.0) / (bursts as f32 - 1.0).max(1.0),
                    y: 50.0 + (k as f32 * 47.0) % 90.0,
                    vx: a.cos() * 16.0,
                    vy: 10.0,
                    life: 2.2 + self.rng.f32() * 0.4,
                    max: 2.6,
                    size: 18.0 + tier as f32 * 3.0,
                    rot: a,
                    col: fw_cols[k % fw_cols.len()],
                    _tier: tier,
                });
            }
        }
        if tier >= 4 {
            for _ in 0..80 {
                self.push_confetti(BOWL_X, self.floor - 220.0, tier);
            }
        }
    }

    fn push_confetti(&mut self, x: f32, y: f32, tier: usize) {
        let cols = [
            GOLD,
            rgba(255, 80, 110, 1.0),
            rgba(255, 220, 80, 1.0),
            rgba(255, 255, 255, 1.0),
            rgba(120, 200, 255, 1.0),
            rgba(255, 160, 60, 1.0),
        ];
        let a = self.rng.f32() * TAU;
        let sp = 40.0 + self.rng.f32() * (90.0 + tier as f32 * 30.0);
        self.particles.push(Fx {
            kind: 6,
            x: x + (self.rng.f32() - 0.5) * 80.0,
            y: y + (self.rng.f32() - 0.5) * 40.0,
            vx: a.cos() * sp,
            vy: a.sin() * sp * 0.4 - 40.0 - self.rng.f32() * 50.0,
            life: 1.8 + self.rng.f32() * 1.0,
            max: 2.8,
            size: 5.0 + self.rng.f32() * 5.0,
            rot: self.rng.f32() * TAU,
            col: cols[self.rng.usize(0..cols.len())],
            _tier: tier,
        });
    }

    fn spawn_luxury_ambience(&mut self, i: usize, dt: f32) {
        let tier = self.shows[i].tier;
        let x = self.shows[i].x;
        let g = self.floor;
        if tier >= 2 {
            // sparkle trail while approaching / standing
            let rate = 8.0 + tier as f32 * 6.0;
            if self.rng.f32() < rate * dt {
                self.particles.push(Fx {
                    kind: 2,
                    x: x - 20.0 - self.rng.f32() * 50.0,
                    y: g - 20.0 - self.rng.f32() * 70.0,
                    vx: -10.0 + self.rng.f32() * 8.0,
                    vy: -16.0 - self.rng.f32() * 20.0,
                    life: 0.7 + self.rng.f32() * 0.4,
                    max: 1.1,
                    size: 5.0 + self.rng.f32() * 6.0,
                    rot: 0.0,
                    col: if tier >= 3 {
                        GOLD
                    } else {
                        rgba(255, 230, 160, 1.0)
                    },
                    _tier: tier,
                });
            }
        }
        if tier >= 3 {
            let rate = 16.0 + if tier >= 4 { 22.0 } else { 8.0 };
            if self.rng.f32() < rate * dt {
                self.particles.push(Fx {
                    kind: 5,
                    x: self.rng.f32() * VW,
                    y: -10.0 - self.rng.f32() * 40.0,
                    vx: (self.rng.f32() - 0.5) * 30.0,
                    vy: 22.0 + self.rng.f32() * 30.0,
                    life: 3.0 + self.rng.f32() * 1.4,
                    max: 4.4,
                    size: 11.0 + self.rng.f32() * 8.0,
                    rot: self.rng.f32() * TAU,
                    col: if self.rng.f32() < 0.35 {
                        GOLD
                    } else if self.rng.f32() < 0.5 {
                        rgba(255, 120, 150, 1.0)
                    } else {
                        rgba(255, 190, 200, 1.0)
                    },
                    _tier: tier,
                });
            }
        }
    }

    fn spawn_luxury_finale(&mut self, tier: usize, dt: f32) {
        if tier < 3 {
            return;
        }
        let fw = if tier >= 4 { 12.0 } else { 5.0 };
        if self.rng.f32() < fw * dt {
            let bx = 80.0 + self.rng.f32() * (VW - 160.0);
            let by = 36.0 + self.rng.f32() * 140.0;
            let cols = [
                GOLD,
                rgba(220, 24, 40, 1.0),
                rgba(255, 140, 30, 1.0),
                rgba(255, 50, 80, 1.0),
                rgba(255, 210, 50, 1.0),
            ];
            let col = cols[self.rng.usize(0..cols.len())];
            self.particles.push(Fx {
                kind: 7,
                x: bx,
                y: by,
                vx: 0.0,
                vy: 8.0,
                life: 1.0 + self.rng.f32() * 0.4,
                max: 1.4,
                size: 18.0 + tier as f32 * 4.0,
                rot: self.rng.f32() * TAU,
                col,
                _tier: tier,
            });
            let sparks = if tier >= 4 { 16 } else { 10 };
            for k in 0..sparks {
                let a = k as f32 / sparks as f32 * TAU + self.rng.f32() * 0.25;
                let sp = 55.0 + self.rng.f32() * 90.0;
                self.particles.push(Fx {
                    kind: 2,
                    x: bx,
                    y: by,
                    vx: a.cos() * sp,
                    vy: a.sin() * sp,
                    life: 0.7 + self.rng.f32() * 0.45,
                    max: 1.15,
                    size: 5.0 + self.rng.f32() * 5.0,
                    rot: 0.0,
                    col: cols[k % cols.len()],
                    _tier: tier,
                });
            }
        }
        if tier >= 4 && self.rng.f32() < 28.0 * dt {
            let cx0 = 200.0 + self.rng.f32() * 680.0;
            let cy0 = 20.0 + self.rng.f32() * 80.0;
            self.push_confetti(cx0, cy0, tier);
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
                3 | 7 => {
                    p.vy += 70.0 * dt;
                    p.vx *= (-0.6 * dt).exp();
                }
                5 => {
                    p.vx += (self.time * 2.4 + p.rot).sin() * 28.0 * dt;
                    p.vy += 18.0 * dt;
                    p.rot += dt * 1.6;
                }
                6 => {
                    p.vy += 40.0 * dt;
                    p.rot += dt * 5.0;
                }
                _ => {}
            }
        }
        self.particles.retain(|p| p.life > 0.0);
        if self.particles.len() > 1800 {
            let n = self.particles.len() - 1800;
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
        let front = self.shows.front().filter(|s| s.front);
        if let Some(s) = front {
            let reveal = reveal_dur(s.tier, reveal_base);
            draw_ceremony_back(cx, s, reveal);
        }
        self.draw_fx(cx, true);
        let pour = front.and_then(|s| {
            let reveal = reveal_dur(s.tier, reveal_base);
            if s.phase(reveal) == "pour" {
                Some((
                    treat_for(s.tier),
                    ((s.t - s.walk_dur) / s.pour_d()).clamp(0.0, 1.0),
                ))
            } else {
                None
            }
        });
        let treat_tier = front.map(|s| s.tier).unwrap_or(0);
        // bowl / treat station is a ground prop (fades with the transparent-mode props setting)
        draw_treat_station(
            props,
            self.bowl_food,
            self.bowl_treat,
            pour,
            treat_tier,
            cx.time,
        );
        if let Some(s) = front {
            let reveal = reveal_dur(s.tier, reveal_base);
            draw_ceremony_mid(cx, s, reveal);
        }
        let cap = max_visible.clamp(1, 12);
        let mut vis: Vec<&CatShow> = self.shows.iter().take(cap).collect();
        vis.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for s in vis {
            let reveal = reveal_dur(s.tier, reveal_base);
            let pose = pose_of(s, reveal, cx.time);
            let alpha = if s.phase(reveal) == "leave" {
                1.0 - smooth((s.t - s.reveal_end(reveal)) / s.leave_d())
            } else {
                1.0
            };
            // palanquin/throne under the royal cat (drawn per-cat so it tracks x)
            if s.front && s.tier >= 4 {
                draw_palanquin(cx, s.x, cx.ground(), s, reveal, alpha);
            }
            let look = look_drawn(s);
            let head_top = cat_head_top(s.x, cx.ground(), &look, &pose);
            if alpha > 0.02 {
                // readability halo (transparent mode only)
                let g = cx.ground();
                cx.halo(s.x, (head_top + g) / 2.0, 72.0 * look.scale, (g - head_top) / 2.0 + 12.0);
            }
            draw_cat(cx, s.x, cx.ground(), &look, &pose, alpha);
            // palanquin lifts the royal cat; name tag already accounts for crown
            let tag_lift = if s.front && s.tier >= 4 {
                28.0 + 16.0 * ceremony_scale(s.tier).min(1.5)
            } else {
                0.0
            };
            draw_cat_name_tag(cx, s, head_top - tag_lift, alpha);
        }
        if let Some(s) = front {
            let reveal = reveal_dur(s.tier, reveal_base);
            draw_ceremony_front(cx, s, reveal);
        }
        self.draw_fx(cx, false);
        if front.map(|s| s.tier >= 3).unwrap_or(false) {
            // keep the corner plant, but don't let it hide a feast
        } else {
            draw_foreground(cx, scene);
        }
        if let Some(s) = front {
            let reveal = reveal_dur(s.tier, reveal_base);
            draw_screen_shine(cx, s, reveal);
        }
    }

    pub fn draw_overlay(&self, cx: &Cx, thanks_title: &str, reveal_base: f32) {
        let Some(s) = self.shows.front() else { return };
        if !s.front {
            return;
        }
        let reveal = reveal_dur(s.tier, reveal_base);
        let rt = s.t - s.walk_dur - s.pour_d() - s.eat_d();
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
                5 if !behind => draw_petal(cx, p.x, p.y, p.size, p.rot, with_a(p.col, a * 0.95)),
                6 if !behind => draw_confetti_bit(cx, p.x, p.y, p.size, p.rot, with_a(p.col, a)),
                7 if !behind => draw_firework_burst(cx, p.x, p.y, p.size, p.rot, p.col, a),
                _ => {}
            }
        }
    }
}

/// Saturated gold / crimson / orange burst — no white bloom wash.
fn draw_firework_burst(cx: &Cx, x: f32, y: f32, size: f32, rot: f32, col: Color32, a: f32) {
    let rays = 12;
    for i in 0..rays {
        let ang = rot + i as f32 / rays as f32 * TAU;
        let (s, c) = ang.sin_cos();
        let len = size * (1.65 + 0.45 * (i as f32 * 1.7).sin());
        let x2 = x + c * len;
        let y2 = y + s * len;
        cx.seg(
            (x, y),
            (x2, y2),
            (2.2 + size * 0.05).min(4.4),
            with_a(col, a),
        );
        cx.circle(x2, y2, size * 0.16, with_a(col, a * 0.95));
        let tip = if i % 2 == 0 {
            GOLD
        } else {
            rgba(220, 24, 40, 1.0)
        };
        cx.circle(x2, y2, size * 0.08, with_a(tip, a));
    }
    cx.glow(x, y, size * 2.4, col, 0.42 * a);
    cx.circle(x, y, size * 0.36, with_a(col, a));
    cx.circle(x, y, size * 0.16, with_a(GOLD, a));
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
            let royal_ride = s.tier >= 4 && ph == "walk";
            let spd = if ph == "leave" { 11.0 } else { 10.0 };
            CatPose {
                walk: if royal_ride { 0.0 } else { 1.0 },
                phase: s.qage * spd,
                facing,
                eat: 0.0,
                happy: if ph == "leave" || royal_ride {
                    0.75
                } else {
                    0.15
                },
                blink,
                sit: if royal_ride { 1.0 } else { 0.0 },
                hop: if royal_ride {
                    let lift = 16.0 + 8.0 * ceremony_scale(s.tier).min(1.55);
                    lift + (s.t * 3.2).sin() * 3.0
                } else if s.tier >= 3 && ph == "walk" {
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

    // royal cape (behind the body, after the tail)
    if look.cape {
        let bw_c = 36.0 * k * fluff;
        let bh_c = if sit > 0.5 {
            24.0 * k * fluff
        } else {
            28.0 * k * fluff
        };
        let by_c = body_y + sit * 4.0 * k;
        let sway = (cx.time * 2.2 + look.seed * 3.0).sin() * 8.0 * k;
        let cape = with_a(rgba(140, 16, 40, 1.0), alpha);
        let trim = with_a(GOLD, alpha);
        let c0x = x - 6.0 * k * f;
        let c0y = by_c - bh_c * 0.15;
        cx.poly(
            &[
                (c0x, c0y),
                (c0x - 20.0 * k * f, c0y + 10.0 * k),
                (c0x - 42.0 * k * f + sway, c0y + 56.0 * k),
                (c0x - 6.0 * k * f + sway * 0.4, c0y + 64.0 * k),
                (c0x + 26.0 * k * f, c0y + 54.0 * k),
                (c0x + 18.0 * k * f, c0y + 10.0 * k),
            ],
            cape,
        );
        cx.polyline(
            &[
                (c0x - 20.0 * k * f, c0y + 10.0 * k),
                (c0x - 42.0 * k * f + sway, c0y + 56.0 * k),
                (c0x - 6.0 * k * f + sway * 0.4, c0y + 64.0 * k),
                (c0x + 26.0 * k * f, c0y + 54.0 * k),
            ],
            2.6 * k,
            trim,
        );
        cx.circle(c0x + 2.0 * k * f, c0y, 5.5 * k, trim);
        cx.circle(
            c0x + 2.0 * k * f,
            c0y,
            2.4 * k,
            with_a(rgba(255, 80, 110, 1.0), alpha),
        );
        let _ = bw_c;
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

    // Persian ruff (tiers 4–5)
    if fluff >= 1.35 {
        for (dx, dy, rr) in [
            (-22.0, -2.0, 14.0),
            (22.0, -2.0, 14.0),
            (-16.0, 8.0, 12.0),
            (16.0, 8.0, 12.0),
            (0.0, 12.0, 13.0),
            (-10.0, -8.0, 11.0),
            (12.0, -8.0, 11.0),
        ] {
            cx.circle(bx + dx * k, by + dy * k, rr * k, fur);
            cx.circle(
                bx + dx * k + 2.0 * k,
                by + dy * k - 2.0 * k,
                rr * 0.55 * k,
                light,
            );
        }
    }

    // front loaf or standing chest
    if sit < 0.5 {
        paw(x + 8.0 * k * f - step * 8.0 * k, -step * 0.6, 2.0 * k * f);
        paw(x - 4.0 * k * f + step * 8.0 * k, step * 0.6, -2.0 * k * f);
    }

    // collar — ribbon at tier 3, jeweled at 4–5
    if look.collar {
        let cy = by - bh * 0.7;
        if look.jewels {
            // wide gold jeweled collar
            cx.ellipse(x, cy, 24.0 * k, 8.0 * k, with_a(GOLD_D, alpha));
            cx.ellipse(x, cy, 22.0 * k, 6.5 * k, with_a(GOLD, alpha));
            for (dx, col) in [
                (-14.0, rgba(80, 200, 255, 1.0)),
                (-5.0, rgba(255, 80, 120, 1.0)),
                (5.0, rgba(180, 90, 255, 1.0)),
                (14.0, rgba(80, 255, 180, 1.0)),
            ] {
                cx.circle(x + dx * k, cy, 3.6 * k, with_a(col, alpha));
                cx.circle(
                    x + dx * k - 0.8 * k,
                    cy - 0.8 * k,
                    1.3 * k,
                    with_a(Color32::WHITE, alpha),
                );
            }
            // hanging jewel
            let jx = x + 2.0 * k * f;
            let jy = cy + 14.0 * k + (pose.phase * 2.0).sin() * 1.5 * k;
            cx.seg(
                (x, cy + 4.0 * k),
                (jx, jy - 4.0 * k),
                1.6 * k,
                with_a(GOLD, alpha),
            );
            cx.poly(
                &[
                    (jx, jy - 5.0 * k),
                    (jx + 6.0 * k, jy + 2.0 * k),
                    (jx, jy + 8.0 * k),
                    (jx - 6.0 * k, jy + 2.0 * k),
                ],
                with_a(rgba(80, 200, 255, 1.0), alpha),
            );
            cx.glow(jx, jy, 16.0 * k, GOLD, 0.35 * alpha);
        } else if look.bow {
            // ribbon collar with trailing tails (tier 3)
            let rib = with_a(rgba(200, 30, 60, 1.0), alpha);
            cx.ellipse(x, cy, 21.0 * k, 6.5 * k, rib);
            cx.ellipse(
                x,
                cy,
                18.0 * k,
                3.2 * k,
                with_a(rgba(255, 200, 210, 1.0), 0.55 * alpha),
            );
            let sway = (cx.time * 3.0 + look.seed).sin() * 5.0 * k;
            cx.poly(
                &[
                    (x + 8.0 * k, cy),
                    (x + 28.0 * k + sway, cy + 18.0 * k),
                    (x + 16.0 * k + sway, cy + 20.0 * k),
                    (x + 10.0 * k, cy + 4.0 * k),
                ],
                rib,
            );
            cx.poly(
                &[
                    (x + 8.0 * k, cy),
                    (x + 24.0 * k - sway, cy + 22.0 * k),
                    (x + 14.0 * k - sway, cy + 24.0 * k),
                    (x + 8.0 * k, cy + 4.0 * k),
                ],
                rib,
            );
            cx.circle(x + 6.0 * k, cy + 2.0 * k, 5.0 * k, with_a(SILVER, alpha));
            cx.circle(
                x + 6.0 * k,
                cy + 2.0 * k,
                2.2 * k,
                with_a(Color32::WHITE, alpha),
            );
        } else {
            cx.ellipse(
                x,
                cy,
                20.0 * k,
                6.0 * k,
                with_a(rgba(200, 40, 50, 1.0), alpha),
            );
            let bell_x = x + 2.0 * k * f;
            let bell_y = cy + 10.0 * k + (pose.phase * 2.0).sin() * 2.0 * k * pose.walk;
            cx.circle(bell_x, bell_y, 6.0 * k, with_a(GOLD, alpha));
            cx.circle(bell_x, bell_y + 1.5 * k, 2.2 * k, with_a(GOLD_D, alpha));
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

    // crown — taller, jeweled, glowing (tier 5)
    if look.crown {
        let cy = hy - hr * 0.92;
        cx.glow(hx, cy - 10.0 * k, 52.0 * k, GOLD, 0.5 * alpha);
        cx.rect(
            hx - 26.0 * k,
            cy,
            hx + 26.0 * k,
            cy + 10.0 * k,
            3.0,
            with_a(GOLD_D, alpha),
        );
        cx.rect(
            hx - 24.0 * k,
            cy + 1.0 * k,
            hx + 24.0 * k,
            cy + 8.0 * k,
            2.0,
            with_a(GOLD, alpha),
        );
        let pts = [
            (-22.0, 14.0),
            (-11.0, 22.0),
            (0.0, 30.0),
            (11.0, 22.0),
            (22.0, 14.0),
        ];
        for (dx, hgt) in pts {
            cx.poly(
                &[
                    (hx + dx * k - 6.0 * k, cy + 2.0 * k),
                    (hx + dx * k + 6.0 * k, cy + 2.0 * k),
                    (hx + dx * k, cy - hgt * k),
                ],
                with_a(GOLD, alpha),
            );
            cx.circle(
                hx + dx * k,
                cy - hgt * k,
                3.4 * k,
                with_a(rgba(255, 80, 120, 1.0), alpha),
            );
            cx.circle(
                hx + dx * k - 0.8 * k,
                cy - hgt * k - 0.8 * k,
                1.2 * k,
                with_a(Color32::WHITE, alpha),
            );
        }
        cx.circle(
            hx,
            cy - 32.0 * k,
            6.0 * k,
            with_a(rgba(255, 60, 100, 1.0), alpha),
        );
        cx.circle(hx, cy - 32.0 * k, 2.6 * k, with_a(Color32::WHITE, alpha));
        cx.glow_add(hx, cy - 20.0 * k, 40.0 * k, GOLD, 0.45 * alpha);
    } else if look.jewels {
        cx.glow(hx, hy, hr * 1.8, GOLD, 0.22 * alpha);
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

pub fn draw_treat_station(
    cx: &Cx,
    food: f32,
    treat: Treat,
    pour: Option<(Treat, f32)>,
    tier: usize,
    time: f32,
) {
    let x = treat_x(tier);
    match tier {
        4 => draw_feast_tower(cx, x, food, treat, pour, time),
        3 => draw_gold_bowl_on_cushion(cx, x, food, treat, pour, time),
        2 => draw_silver_bowl(cx, x, food, treat, pour, time),
        _ => draw_bowl_at(cx, x, food, treat, pour),
    }
}

pub fn draw_bowl(cx: &Cx, food: f32, treat: Treat, pour: Option<(Treat, f32)>) {
    draw_bowl_at(cx, BOWL_X, food, treat, pour);
}

fn draw_bowl_at(cx: &Cx, x: f32, food: f32, treat: Treat, pour: Option<(Treat, f32)>) {
    let g = cx.ground();
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
    // paw / jewel corners
    for (px, py) in [
        (x0 + 22.0, top + 22.0),
        (x1 - 22.0, top + 22.0),
        (x0 + 22.0, top + hgt - 22.0),
        (x1 - 22.0, top + hgt - 22.0),
    ] {
        if tier >= 3 {
            cx.circle(px, py, 9.0, with_a(GOLD, alpha));
            cx.circle(px, py, 4.5, with_a(rgba(255, 80, 120, 1.0), alpha));
            cx.circle(px - 1.5, py - 1.5, 1.6, with_a(Color32::WHITE, alpha));
        } else {
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
    if tier >= 3 {
        // heavier gold frame from tier 4; crown cap at tier 5
        cx.rect_line(
            x0 - 8.0,
            top - 8.0,
            x1 + 8.0,
            top + hgt + 8.0,
            26.0,
            5.0,
            with_a(GOLD, alpha),
        );
        if tier >= 4 {
            cx.rect_line(
                x0 - 14.0,
                top - 14.0,
                x1 + 14.0,
                top + hgt + 14.0,
                30.0,
                2.5,
                with_a(GOLD_D, 0.85 * alpha),
            );
            let cx0 = (x0 + x1) * 0.5;
            let cy = top - 8.0;
            cx.rect(
                cx0 - 28.0,
                cy - 6.0,
                cx0 + 28.0,
                cy + 8.0,
                3.0,
                with_a(GOLD, alpha),
            );
            cx.rect(
                cx0 - 48.0,
                cy - 8.0,
                cx0 + 48.0,
                cy + 10.0,
                3.0,
                with_a(GOLD_D, alpha),
            );
            for (dx, h) in [
                (-38.0, 16.0),
                (-20.0, 24.0),
                (0.0, 34.0),
                (20.0, 24.0),
                (38.0, 16.0),
            ] {
                cx.poly(
                    &[
                        (cx0 + dx - 8.0, cy),
                        (cx0 + dx + 8.0, cy),
                        (cx0 + dx, cy - h),
                    ],
                    with_a(GOLD, alpha),
                );
            }
            cx.circle(cx0, cy - 36.0, 6.0, with_a(rgba(255, 80, 110, 1.0), alpha));
            cx.circle(cx0, cy - 36.0, 2.4, with_a(Color32::WHITE, alpha));
            cx.glow(cx0, cy - 10.0, 90.0, GOLD, 0.5 * alpha);
            // gold pillars at the sides
            for side in [-1.0, 1.0] {
                let px = cx0 + side * (wdt * 0.5 + 6.0);
                cx.rect(
                    px - 7.0,
                    top + 24.0,
                    px + 7.0,
                    top + hgt - 24.0,
                    3.0,
                    with_a(GOLD, alpha),
                );
                cx.circle(px, top + 22.0, 9.0, with_a(GOLD, alpha));
                cx.circle(px, top + 22.0, 4.0, with_a(rgba(255, 80, 110, 1.0), alpha));
                cx.circle(px, top + hgt - 22.0, 9.0, with_a(GOLD, alpha));
                cx.circle(
                    px,
                    top + hgt - 22.0,
                    4.0,
                    with_a(rgba(255, 80, 110, 1.0), alpha),
                );
            }
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
    let hgt = [340.0, 348.0, 360.0, 380.0, 400.0][tier.min(4)];
    let wdt = [640.0, 690.0, 760.0, 840.0, 920.0][tier.min(4)];
    let cxx = 540.0;
    let pad = [20.0, 24.0, 28.0, 36.0, 52.0][tier.min(4)];
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
    let title_size = [40.0, 42.0, 44.0, 48.0, 52.0][tier.min(4)];
    if tier >= 4 {
        cx.glow(cxx, top + 80.0, 260.0, GOLD, 0.5 * ca);
        cx.text_o(
            cxx,
            top + 80.0,
            Align2::CENTER_CENTER,
            &title,
            title_size,
            with_a(GOLD, ca),
            with_a(rgba(80, 30, 10, 0.9), ca),
            true,
        );
    } else {
        cx.text_o(
            cxx,
            top + 74.0,
            Align2::CENTER_CENTER,
            &title,
            title_size,
            with_a(rgba(150, 50, 50, 1.0), ca),
            with_a(Color32::WHITE, ca),
            true,
        );
    }
    // hearts beside the name, fancier with tier
    let nhearts = [1, 1, 2, 4, 6][tier.min(4)];
    for i in 0..nhearts {
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let k = (i / 2) as f32;
        let hx = cxx + side * (wdt * 0.36 + k * 18.0);
        let hy = top + if tier >= 4 { 80.0 } else { 74.0 } + (k - 0.3) * 6.0;
        let pulse = 0.85 + 0.15 * (cx.time * 3.0 + i as f32).sin();
        draw_heart(
            cx,
            hx,
            hy,
            (11.0 + tier as f32 * 0.8) * pulse,
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
        cx.glow(cxx, top + 80.0, 240.0, GOLD, 0.35 * ca);
        // soft rays behind the card (screen-wide gold, kept readable)
        if cx.soft {
            let rays = 14;
            for i in 0..rays {
                let a = i as f32 / rays as f32 * TAU + cx.time * 0.35;
                let len = 280.0 + 40.0 * (cx.time * 2.0 + i as f32).sin();
                let (s, c) = a.sin_cos();
                let ox = cxx;
                let oy = top + 70.0;
                cx.poly(
                    &[
                        (ox - s * 6.0, oy + c * 6.0),
                        (ox + s * 6.0, oy - c * 6.0),
                        (ox + c * len, oy + s * len),
                    ],
                    rgba(255, 210, 80, 0.10 * ca),
                );
            }
        }
    }
}

fn draw_petal(cx: &Cx, x: f32, y: f32, s: f32, rot: f32, c: Color32) {
    let (sn, cs) = rot.sin_cos();
    let p = |dx: f32, dy: f32| (x + dx * cs - dy * sn, y + dx * sn + dy * cs);
    cx.poly(
        &[
            p(0.0, -s),
            p(s * 0.7, 0.0),
            p(0.0, s * 0.85),
            p(-s * 0.7, 0.0),
        ],
        c,
    );
}

fn draw_confetti_bit(cx: &Cx, x: f32, y: f32, s: f32, rot: f32, c: Color32) {
    let (sn, cs) = rot.sin_cos();
    let hw = s * 0.7;
    let hh = s * 0.35;
    let p = |dx: f32, dy: f32| (x + dx * cs - dy * sn, y + dx * sn + dy * cs);
    cx.poly(&[p(-hw, -hh), p(hw, -hh), p(hw, hh), p(-hw, hh)], c);
}

fn carpet_reach(s: &CatShow, reveal: f32) -> f32 {
    let ph = s.phase(reveal);
    match ph {
        "walk" => ease_out_cubic((s.t / s.walk_dur.max(0.2)).clamp(0.0, 1.0)),
        "leave" => 1.0 - smooth((s.t - s.reveal_end(reveal)) / s.leave_d().max(0.2)),
        _ => 1.0,
    }
}

/// Red carpet + gold rope posts, unrolling toward the bowl. Solid enough for transparent mode.
fn draw_carpet_runway(cx: &Cx, reach: f32, royal: bool, alpha: f32) {
    if reach <= 0.02 || alpha <= 0.0 {
        return;
    }
    let g = cx.ground();
    let start = 40.0;
    let end_full = BOWL_X + 150.0;
    let end = start + (end_full - start) * ease_out_cubic(reach);
    let col = if royal {
        rgba(150, 10, 28, 1.0)
    } else {
        rgba(170, 20, 40, 1.0)
    };
    // wide runner
    cx.poly(
        &[
            (start, g - 18.0),
            (end - 14.0, g - 18.0),
            (end + 6.0, g + 16.0),
            (start, g + 16.0),
        ],
        with_a(col, 0.96 * alpha),
    );
    // gold borders
    cx.poly(
        &[
            (start, g - 20.0),
            (end - 16.0, g - 20.0),
            (end - 10.0, g - 14.0),
            (start, g - 14.0),
        ],
        with_a(GOLD, 0.95 * alpha),
    );
    cx.poly(
        &[
            (start, g + 12.0),
            (end + 4.0, g + 12.0),
            (end + 8.0, g + 18.0),
            (start, g + 18.0),
        ],
        with_a(GOLD, 0.95 * alpha),
    );
    // diamond motifs
    let mut x = start + 50.0;
    while x < end - 30.0 {
        cx.poly(
            &[
                (x, g - 8.0),
                (x + 10.0, g - 1.0),
                (x, g + 6.0),
                (x - 10.0, g - 1.0),
            ],
            with_a(GOLD, 0.8 * alpha),
        );
        x += 64.0;
    }
    if reach < 0.99 {
        cx.ellipse(end, g - 2.0, 18.0, 16.0, with_a(shade(col, 0.8), alpha));
        cx.ellipse(end, g - 2.0, 14.0, 12.0, with_a(col, alpha));
        cx.ring(end, g - 2.0, 16.0, 2.0, with_a(GOLD, alpha));
    }
    // gold rope posts along the sides
    if royal {
        let mut px = start + 30.0;
        let mut prev_l = None;
        let mut prev_r = None;
        while px < end - 10.0 {
            for (side, prev) in [(-1.0, &mut prev_l), (1.0, &mut prev_r)] {
                let x = px;
                let y = g - 4.0 + side * 28.0;
                if let Some(q) = *prev {
                    cx.seg(q, (x, y - 28.0), 2.6, with_a(GOLD, 0.9 * alpha));
                }
                // post
                cx.rect(
                    x - 4.0,
                    y - 38.0,
                    x + 4.0,
                    y + 6.0,
                    2.0,
                    with_a(GOLD_D, alpha),
                );
                cx.rect(
                    x - 3.0,
                    y - 36.0,
                    x + 3.0,
                    y + 4.0,
                    1.5,
                    with_a(GOLD, alpha),
                );
                cx.circle(x, y - 40.0, 7.0, with_a(GOLD, alpha));
                cx.circle(x, y - 40.0, 3.0, with_a(rgba(255, 80, 110, 1.0), alpha));
                *prev = Some((x, y - 28.0));
            }
            px += 90.0;
        }
    }
}

fn draw_ceremony_back(cx: &Cx, s: &CatShow, reveal: f32) {
    let a = if s.phase(reveal) == "leave" {
        1.0 - smooth((s.t - s.reveal_end(reveal)) / s.leave_d())
    } else {
        1.0
    };
    if s.tier >= 3 {
        let reach = carpet_reach(s, reveal);
        draw_carpet_runway(cx, reach, s.tier >= 3, a);
    }
    if s.tier >= 4 && cx.soft {
        // warm gold light wash near the feast (keeps the streamer visible — no full-screen veil)
        let sc = ceremony_scale(s.tier);
        cx.glow(
            treat_x(s.tier),
            cx.ground() - 80.0 * sc,
            260.0 * sc,
            GOLD,
            0.22 * a,
        );
        if s.tier >= 4 {
            cx.glow(s.x, cx.ground() - 90.0 * sc, 220.0 * sc, GOLD, 0.2 * a);
        }
    }
}

fn draw_ceremony_mid(cx: &Cx, s: &CatShow, reveal: f32) {
    let a = if s.phase(reveal) == "leave" {
        1.0 - smooth((s.t - s.reveal_end(reveal)) / s.leave_d())
    } else {
        1.0
    };
    if s.tier < 3 {
        return;
    }
    // attendants flank the throne: one left of the hero, one right (before the treat tower)
    let g = cx.ground();
    let ph = s.phase(reveal);
    let bob = (cx.time * 3.0).sin() * 2.0;
    if s.tier >= 3 {
        let k = attendant_scale(s.tier);
        let flank = 88.0 + 70.0 * ceremony_scale(s.tier).sqrt();
        let (ax, bx) = (s.x - flank, s.x + flank);
        if s.tier >= 4 && ph == "walk" {
            // they walk with the palanquin as bearers (also drawn in palanquin)
        } else {
            draw_attendant_kitten(
                cx,
                ax,
                g,
                1.0,
                if s.tier >= 4 { "fan" } else { "sit" },
                cx.time,
                a,
                rgba(240, 220, 190, 1.0),
                k,
            );
            draw_attendant_kitten(
                cx,
                bx,
                g,
                -1.0,
                if s.tier >= 4 { "fan" } else { "bow" },
                cx.time + 0.7,
                a,
                rgba(50, 44, 48, 1.0),
                k,
            );
            let _ = bob;
        }
    }
}

fn draw_ceremony_front(cx: &Cx, s: &CatShow, reveal: f32) {
    let a = if s.phase(reveal) == "leave" {
        1.0 - smooth((s.t - s.reveal_end(reveal)) / s.leave_d())
    } else {
        1.0
    };
    if s.tier >= 4 {
        // light rays over the feast during eat/card
        let ph = s.phase(reveal);
        if matches!(ph, "pour" | "eat" | "card") && cx.soft {
            let x = treat_x(s.tier);
            let y = 20.0;
            let n = if s.tier >= 4 { 16 } else { 10 };
            let pulse = 0.55 + 0.45 * (cx.time * 1.8).sin().abs();
            for i in 0..n {
                let a0 = -0.9 + i as f32 / n as f32 * 1.8;
                let len = 280.0 + 50.0 * (cx.time * 2.0 + i as f32).sin();
                let (sn, cs) = a0.sin_cos();
                cx.poly(
                    &[
                        (x - 8.0, y),
                        (x + 8.0, y),
                        (x + cs * 18.0 + sn * len, y + cs * len),
                    ],
                    rgba(255, 215, 90, 0.11 * a * pulse),
                );
            }
            cx.glow_add(x, y + 10.0, 160.0, GOLD, 0.35 * a * pulse);
        }
    }
}

fn draw_screen_shine(cx: &Cx, s: &CatShow, _reveal: f32) {
    if s.tier < 4 || s.burst_at < 0.0 {
        return;
    }
    let rt = s.t - s.burst_at;
    if rt < 0.0 || rt > 0.45 {
        return;
    }
    let flash = (1.0 - (rt / 0.45).clamp(0.0, 1.0)).powf(1.6);
    // brief gold shine — strong but short, won't hide the streamer on a still
    cx.glow_add(VW * 0.5, 80.0, 520.0, GOLD, 0.55 * flash);
    cx.glow_add(
        BOWL_X,
        cx.ground() - 80.0,
        300.0,
        rgba(255, 240, 180, 1.0),
        0.4 * flash,
    );
    if cx.soft {
        cx.vgradient(
            -20.0,
            -20.0,
            VW + 20.0,
            220.0,
            rgba(255, 230, 140, 0.22 * flash),
            rgba(255, 230, 140, 0.0),
        );
    }
}

fn draw_palanquin(cx: &Cx, x: f32, g: f32, s: &CatShow, reveal: f32, alpha: f32) {
    let ph = s.phase(reveal);
    let carried = ph == "walk";
    let sc = ceremony_scale(s.tier);
    let wsc = sc.min(1.7);
    // Width scales fully (capped so the feast still sits clear); canopy height is
    // reduced during the thank-you card so they don't collide.
    let hsc = if ph == "card" {
        1.05
    } else {
        1.0 + (sc - 1.0) * 0.72
    };
    let bob = if carried {
        (s.t * 3.2).sin() * 3.0
    } else {
        0.0
    };
    let base = g - if carried { 8.0 } else { 4.0 } - bob;
    // platform
    cx.ellipse(
        x,
        g - 2.0,
        78.0 * wsc,
        14.0 * wsc,
        rgba(0, 0, 0, 0.28 * alpha),
    );
    cx.rect(
        x - 70.0 * wsc,
        base - 18.0,
        x + 70.0 * wsc,
        base + 6.0,
        8.0,
        with_a(GOLD_D, alpha),
    );
    cx.rect(
        x - 66.0 * wsc,
        base - 16.0,
        x + 66.0 * wsc,
        base + 2.0,
        6.0,
        with_a(GOLD, alpha),
    );
    // velvet cushion
    cx.ellipse(
        x,
        base - 20.0,
        48.0 * wsc,
        16.0 * wsc,
        with_a(CRIMSON, alpha),
    );
    cx.ellipse(
        x,
        base - 26.0,
        40.0 * wsc,
        12.0 * wsc,
        with_a(rgba(190, 30, 55, 1.0), alpha),
    );
    cx.ellipse(
        x + 8.0 * wsc,
        base - 28.0,
        16.0 * wsc,
        6.0 * wsc,
        with_a(rgba(255, 180, 190, 0.45), alpha),
    );
    // corner tassels
    for dx in [-62.0 * wsc, 62.0 * wsc] {
        cx.seg(
            (x + dx, base - 8.0),
            (x + dx, base + 10.0),
            2.0,
            with_a(GOLD, alpha),
        );
        cx.circle(
            x + dx,
            base + 12.0,
            4.0 * sc.min(1.4),
            with_a(rgba(255, 80, 110, 1.0), alpha),
        );
    }
    // canopy posts + roof when carried or during pour/eat (throne)
    let posts = [-58.0 * wsc, 58.0 * wsc];
    let post_h = 150.0 * hsc;
    for dx in posts {
        cx.rect(
            x + dx - 4.0 * sc.min(1.4),
            base - post_h,
            x + dx + 4.0 * sc.min(1.4),
            base - 12.0,
            1.5,
            with_a(GOLD, alpha),
        );
        cx.circle(
            x + dx,
            base - post_h - 2.0,
            7.0 * sc.min(1.35),
            with_a(GOLD, alpha),
        );
        cx.circle(
            x + dx,
            base - post_h - 2.0,
            3.0 * sc.min(1.35),
            with_a(rgba(255, 80, 110, 1.0), alpha),
        );
    }
    // tall ceremonial roof / backboard that frames the crown
    let roof_y = 142.0 * hsc;
    let tip_y = 198.0 * hsc;
    cx.poly(
        &[
            (x - 80.0 * wsc, base - roof_y),
            (x + 80.0 * wsc, base - roof_y),
            (x + 62.0 * wsc, base - 176.0 * hsc),
            (x, base - tip_y),
            (x - 62.0 * wsc, base - 176.0 * hsc),
        ],
        with_a(CRIMSON, alpha),
    );
    cx.poly(
        &[
            (x - 72.0 * wsc, base - (roof_y + 4.0)),
            (x + 72.0 * wsc, base - (roof_y + 4.0)),
            (x, base - 186.0 * hsc),
        ],
        with_a(GOLD, 0.95 * alpha),
    );
    cx.circle(
        x,
        base - tip_y - 2.0,
        8.0 * sc.min(1.35),
        with_a(GOLD, alpha),
    );
    cx.circle(
        x,
        base - tip_y - 2.0,
        3.4 * sc.min(1.35),
        with_a(rgba(255, 80, 110, 1.0), alpha),
    );
    cx.glow(x, base - 170.0 * hsc, 80.0 * sc, GOLD, 0.3 * alpha);
    // carrying poles + bearers while walking
    if carried {
        let pole = 130.0 * wsc;
        cx.rect(
            x - pole,
            base - 22.0,
            x + pole,
            base - 16.0,
            2.0,
            with_a(GOLD_D, alpha),
        );
        cx.rect(
            x - pole + 2.0,
            base - 21.0,
            x + pole - 2.0,
            base - 17.0,
            1.5,
            with_a(GOLD, alpha),
        );
        let k = attendant_scale(s.tier);
        let bear = 108.0 * wsc.sqrt();
        draw_attendant_kitten(
            cx,
            x - bear,
            g,
            1.0,
            "carry",
            cx.time,
            alpha,
            rgba(240, 220, 190, 1.0),
            k,
        );
        draw_attendant_kitten(
            cx,
            x + bear,
            g,
            -1.0,
            "carry",
            cx.time + 0.4,
            alpha,
            rgba(50, 44, 48, 1.0),
            k,
        );
    }
}

fn draw_fan(cx: &Cx, x: f32, y: f32, ang: f32, sc: f32, alpha: f32) {
    let (sn, cs) = ang.sin_cos();
    let p = |dx: f32, dy: f32| (x + (dx * cs - dy * sn) * sc, y + (dx * sn + dy * cs) * sc);
    cx.poly(
        &[p(0.0, 0.0), p(22.0, -28.0), p(0.0, -36.0), p(-22.0, -28.0)],
        with_a(GOLD, alpha),
    );
    cx.poly(
        &[p(0.0, -2.0), p(16.0, -24.0), p(0.0, -30.0), p(-16.0, -24.0)],
        with_a(CRIMSON, alpha),
    );
    for k in 0..5 {
        let u = (k as f32 - 2.0) / 2.0;
        cx.seg(
            p(0.0, 0.0),
            p(u * 18.0, -30.0),
            1.2,
            with_a(GOLD, 0.85 * alpha),
        );
    }
    cx.circle(x, y, 3.5 * sc, with_a(GOLD, alpha));
}

fn draw_attendant_kitten(
    cx: &Cx,
    x: f32,
    g: f32,
    face: f32,
    job: &str,
    time: f32,
    alpha: f32,
    fur: Color32,
    k: f32,
) {
    let dark = shade(fur, 0.7);
    let light = shade(fur, 1.25);
    let bob = if job == "carry" {
        (time * 8.0).sin() * 2.0
    } else {
        (time * 1.6).sin() * 1.2
    };
    let by = g - 20.0 * k - bob;
    cx.ellipse(x, g - 3.0, 22.0 * k, 6.0 * k, rgba(0, 0, 0, 0.22 * alpha));
    // body
    cx.ellipse(x, by, 20.0 * k, 14.0 * k, with_a(dark, alpha));
    cx.ellipse(x, by, 18.0 * k, 12.0 * k, with_a(fur, alpha));
    // head
    let hx = x + 6.0 * k * face;
    let hy = by - 16.0 * k;
    // ears
    for side in [-1.0, 1.0] {
        cx.poly(
            &[
                (hx + side * 10.0 * k, hy - 2.0 * k),
                (hx + side * 16.0 * k, hy - 16.0 * k),
                (hx + side * 4.0 * k, hy - 6.0 * k),
            ],
            with_a(fur, alpha),
        );
        cx.poly(
            &[
                (hx + side * 10.0 * k, hy - 4.0 * k),
                (hx + side * 14.0 * k, hy - 12.0 * k),
                (hx + side * 6.0 * k, hy - 6.0 * k),
            ],
            with_a(rgba(255, 170, 180, 1.0), alpha),
        );
    }
    cx.circle(hx, hy, 13.0 * k, with_a(dark, alpha));
    cx.circle(hx, hy, 12.0 * k, with_a(fur, alpha));
    cx.ellipse(
        hx,
        hy + 2.0 * k,
        6.0 * k,
        4.0 * k,
        with_a(light, 0.7 * alpha),
    );
    // cute closed or open eyes
    if job == "bow" {
        for side in [-1.0, 1.0] {
            cx.polyline(
                &[
                    (hx + side * 5.0 * k - 3.0 * k, hy + 1.0 * k),
                    (hx + side * 5.0 * k, hy - 2.0 * k),
                    (hx + side * 5.0 * k + 3.0 * k, hy + 1.0 * k),
                ],
                1.6 * k,
                with_a(rgba(40, 30, 30, 1.0), alpha),
            );
        }
    } else {
        for side in [-1.0, 1.0] {
            cx.circle(
                hx + side * 5.0 * k,
                hy - 1.0 * k,
                2.4 * k,
                with_a(rgba(30, 30, 30, 1.0), alpha),
            );
            cx.circle(
                hx + side * 5.0 * k - 0.7 * k,
                hy - 1.6 * k,
                0.8 * k,
                with_a(Color32::WHITE, alpha),
            );
        }
    }
    cx.poly(
        &[
            (hx, hy + 3.0 * k),
            (hx + 2.4 * k, hy + 5.5 * k),
            (hx - 2.4 * k, hy + 5.5 * k),
        ],
        with_a(rgba(255, 130, 150, 1.0), alpha),
    );
    // tiny gold collar
    cx.ellipse(hx, hy + 10.0 * k, 8.0 * k, 2.6 * k, with_a(GOLD, alpha));
    if job == "fan" {
        let ang = -0.4 + (time * 4.0).sin() * 0.55;
        draw_fan(cx, x + 16.0 * face, by - 8.0, ang, 0.85, alpha);
    }
}

fn pour_stream(cx: &Cx, x: f32, rim_y: f32, pour: Option<(Treat, f32)>) {
    if let Some((tr, u)) = pour {
        let y0 = rim_y - 120.0;
        let y1 = rim_y - 8.0;
        let y = lerp(y0, y1, ease_out_cubic(u));
        cx.rect(
            x - 3.5,
            y0,
            x + 3.5,
            y,
            2.0,
            with_a(food_color(tr, 0.3), 0.85),
        );
        if matches!(tr, Treat::Fish | Treat::Sashimi | Treat::Feast) && u > 0.3 {
            draw_mini_fish(cx, x + 18.0, lerp(y0, y1, ease_in_out(u)), 1.05, 0.25);
        }
    }
}

fn draw_silver_bowl(
    cx: &Cx,
    x: f32,
    food: f32,
    treat: Treat,
    pour: Option<(Treat, f32)>,
    time: f32,
) {
    let g = cx.ground();
    cx.ellipse(x, g - 4.0, 96.0, 20.0, rgba(120, 140, 160, 0.9));
    cx.ellipse(x, g - 4.0, 78.0, 14.0, rgba(170, 188, 205, 0.95));
    let rim_y = g - 36.0;
    cx.ellipse(x, g - 12.0, 58.0, 16.0, SILVER_D);
    cx.ellipse(
        x,
        rim_y + 14.0,
        62.0,
        20.0,
        Color32::from_rgb(168, 184, 200),
    );
    cx.ellipse(x, rim_y, 68.0, 17.0, SILVER);
    cx.ring(x, rim_y, 68.0, 5.0, SILVER_D);
    cx.ring(x, rim_y, 60.0, 2.4, Color32::WHITE);
    // shine
    let sh = 0.45 + 0.25 * (time * 3.0).sin();
    cx.ellipse(x - 16.0, rim_y + 4.0, 10.0, 5.0, rgba(255, 255, 255, sh));
    fill_bowl(cx, x, rim_y, food, treat, 1.05);
    pour_stream(cx, x, rim_y, pour);
}

fn draw_gold_bowl_on_cushion(
    cx: &Cx,
    x: f32,
    food: f32,
    treat: Treat,
    pour: Option<(Treat, f32)>,
    time: f32,
) {
    let g = cx.ground();
    let sc = ceremony_scale(3);
    let csc = 1.28; // cushion slightly tighter than the 1.5× bowl so it clears the cat
                    // velvet cushion
    cx.ellipse(x, g - 2.0, 130.0 * csc, 28.0 * csc, with_a(CRIMSON, 0.98));
    cx.ellipse(
        x,
        g - 12.0 * sc,
        112.0 * csc,
        22.0 * csc,
        rgba(180, 24, 48, 1.0),
    );
    cx.ellipse(
        x + 18.0 * sc,
        g - 16.0 * sc,
        36.0 * sc,
        10.0 * sc,
        rgba(255, 180, 190, 0.4),
    );
    // tassels
    for dx in [-70.0 * sc, 70.0 * sc] {
        cx.circle(x + dx, g - 4.0, 7.0 * sc.min(1.35), GOLD);
        cx.seg((x + dx, g - 2.0), (x + dx, g + 10.0), 2.0, GOLD);
    }
    let rim_y = g - 44.0 * sc;
    cx.glow(x, rim_y, 90.0 * sc, GOLD, 0.35);
    cx.ellipse(x, g - 16.0 * sc, 62.0 * sc, 16.0 * sc, GOLD_D);
    cx.ellipse(x, rim_y + 14.0 * sc, 68.0 * sc, 20.0 * sc, GOLD);
    cx.ellipse(
        x,
        rim_y,
        74.0 * sc,
        18.0 * sc,
        Color32::from_rgb(255, 230, 140),
    );
    cx.ring(x, rim_y, 74.0 * sc, 5.5 * sc.min(1.3), GOLD_D);
    cx.ring(x, rim_y, 66.0 * sc, 2.8, GOLD);
    // gem on rim
    for dx in [-28.0 * sc, 0.0, 28.0 * sc] {
        cx.circle(x + dx, rim_y, 5.0 * sc.min(1.35), rgba(255, 80, 120, 1.0));
        cx.circle(
            x + dx - 1.2,
            rim_y - 1.2,
            1.6 * sc.min(1.35),
            Color32::WHITE,
        );
    }
    let sh = 0.5 + 0.3 * (time * 3.2).sin();
    cx.ellipse(
        x - 18.0 * sc,
        rim_y + 5.0 * sc,
        12.0 * sc,
        6.0 * sc,
        rgba(255, 255, 220, sh),
    );
    fill_bowl(cx, x, rim_y, food, treat, 1.2 * sc);
    pour_stream(cx, x, rim_y, pour);
}

fn fill_bowl(cx: &Cx, x: f32, rim_y: f32, food: f32, treat: Treat, sc: f32) {
    let fill = food.clamp(0.0, 1.0);
    if fill <= 0.02 {
        return;
    }
    let fy = rim_y + 6.0 * sc + (1.0 - fill) * 8.0 * sc;
    let col = match treat {
        Treat::Kibble => rgba(180, 110, 50, 1.0),
        Treat::Can => rgba(200, 120, 70, 1.0),
        Treat::Fish => rgba(240, 200, 130, 1.0),
        Treat::Sashimi => rgba(255, 130, 110, 1.0),
        Treat::Feast => rgba(255, 170, 90, 1.0),
    };
    cx.ellipse(
        x,
        fy,
        42.0 * sc * (0.7 + 0.3 * fill),
        (10.0 * fill + 5.0) * sc,
        col,
    );
    if matches!(treat, Treat::Fish | Treat::Feast | Treat::Sashimi) {
        cx.ellipse(
            x + 8.0 * sc,
            fy - 3.0 * sc,
            18.0 * sc,
            7.0 * sc,
            rgba(255, 220, 160, 1.0),
        );
        cx.ellipse(
            x - 12.0 * sc,
            fy - 1.0 * sc,
            12.0 * sc,
            5.0 * sc,
            rgba(255, 150, 110, 1.0),
        );
    }
}

/// Giant golden fish platter + multi-tier treat tower (tier 5).
fn draw_feast_tower(
    cx: &Cx,
    x: f32,
    food: f32,
    treat: Treat,
    pour: Option<(Treat, f32)>,
    time: f32,
) {
    let g = cx.ground();
    let sc = ceremony_scale(4);
    let pulse = 0.55 + 0.45 * (time * 2.2).sin().abs();
    cx.glow(x, g - 80.0 * sc, 160.0 * sc, GOLD, 0.4 * pulse);
    // pedestal
    cx.ellipse(x, g - 2.0, 100.0 * sc, 20.0 * sc, with_a(CRIMSON, 0.95));
    cx.ellipse(
        x,
        g - 8.0 * sc,
        86.0 * sc,
        16.0 * sc,
        rgba(170, 16, 40, 1.0),
    );
    cx.rect(
        x - 36.0 * sc,
        g - 48.0 * sc,
        x + 36.0 * sc,
        g - 8.0 * sc,
        6.0,
        GOLD_D,
    );
    cx.rect(
        x - 32.0 * sc,
        g - 46.0 * sc,
        x + 32.0 * sc,
        g - 10.0 * sc,
        5.0,
        GOLD,
    );
    // three gold plates, stacked
    let levels = [
        (0.0, 52.0 * sc, 70.0 * sc),
        (-36.0 * sc, 86.0 * sc, 50.0 * sc),
        (0.0, 118.0 * sc, 36.0 * sc),
    ];
    for (i, (dx, h, r)) in levels.iter().enumerate() {
        let px = x + *dx;
        let py = g - *h;
        cx.ellipse(px, py + 8.0 * sc, *r, 12.0 * sc, GOLD_D);
        cx.ellipse(px, py, *r + 4.0 * sc, 11.0 * sc, GOLD);
        cx.ellipse(
            px,
            py,
            *r - 6.0 * sc,
            7.0 * sc,
            Color32::from_rgb(255, 236, 170),
        );
        cx.ring(px, py, *r + 4.0 * sc, 3.0 * sc.min(1.35), GOLD_D);
        // food on each plate
        if food > 0.08 {
            match i {
                0 => {
                    // giant golden fish
                    draw_royal_fish(cx, px - 4.0 * sc, py - 16.0 * sc, 1.85 * sc, -0.12, time);
                    draw_mini_fish(cx, px + 28.0 * sc, py - 2.0 * sc, 0.7 * sc, 0.4);
                    draw_mini_fish(cx, px - 30.0 * sc, py - 1.0 * sc, 0.6 * sc, -0.3);
                }
                1 => {
                    // sashimi + gold orbs
                    for (ox, oy, c) in [
                        (-14.0, -4.0, rgba(255, 120, 100, 1.0)),
                        (0.0, -8.0, rgba(255, 160, 90, 1.0)),
                        (14.0, -4.0, rgba(255, 100, 110, 1.0)),
                    ] {
                        cx.ellipse(px + ox * sc, py + oy * sc, 11.0 * sc, 5.5 * sc, c);
                    }
                    cx.circle(px, py - 16.0 * sc, 6.0 * sc, GOLD);
                }
                _ => {
                    cx.ellipse(
                        px,
                        py - 8.0 * sc,
                        14.0 * sc,
                        8.0 * sc,
                        rgba(255, 200, 80, 1.0),
                    );
                    cx.circle(px, py - 16.0 * sc, 7.0 * sc, rgba(255, 80, 110, 1.0));
                    cx.circle(px, py - 16.0 * sc, 3.0 * sc, Color32::WHITE);
                    cx.glow(px, py - 16.0 * sc, 22.0 * sc, GOLD, 0.4);
                }
            }
        }
    }
    // steam / aroma
    for i in 0..4 {
        let u = ((time * 0.7 + i as f32 * 0.4) % 1.0).abs();
        cx.ellipse(
            x - 16.0 * sc + i as f32 * 10.0 * sc,
            g - 140.0 * sc - u * 30.0 * sc,
            (6.0 + u * 8.0) * sc,
            10.0 * sc,
            rgba(255, 240, 200, 0.28 * (1.0 - u)),
        );
    }
    let _ = treat;
    if let Some((_, u)) = pour {
        pour_stream(cx, x, g - 52.0 * sc, pour);
        cx.glow(x, g - 52.0 * sc - 80.0 * sc, 70.0 * sc, GOLD, 0.3 * u);
    }
}

fn draw_royal_fish(cx: &Cx, x: f32, y: f32, sc: f32, ang: f32, time: f32) {
    let (s, c) = ang.sin_cos();
    let p = |dx: f32, dy: f32| (x + dx * sc * c - dy * sc * s, y + dx * sc * s + dy * sc * c);
    cx.glow(x, y, 40.0 * sc, GOLD, 0.35);
    cx.poly(
        &[p(-28.0, 0.0), p(18.0, -14.0), p(30.0, 0.0), p(18.0, 14.0)],
        GOLD,
    );
    cx.poly(
        &[p(-26.0, 0.0), p(14.0, -10.0), p(24.0, 0.0), p(14.0, 10.0)],
        rgba(255, 180, 80, 1.0),
    );
    cx.poly(
        &[
            p(-28.0, 0.0),
            p(-44.0, -14.0),
            p(-36.0, 0.0),
            p(-44.0, 14.0),
        ],
        GOLD_D,
    );
    cx.circle(
        p(16.0, -3.0).0,
        p(16.0, -3.0).1,
        3.2 * sc,
        rgba(40, 30, 30, 1.0),
    );
    cx.circle(p(15.2, -3.6).0, p(15.2, -3.6).1, 1.2 * sc, Color32::WHITE);
    // glitter scales
    for i in 0..5 {
        let dx = -16.0 + i as f32 * 7.0;
        let tw = 0.6 + 0.4 * (time * 6.0 + i as f32).sin().abs();
        cx.ellipse(
            p(dx, -3.0).0,
            p(dx, -3.0).1,
            3.5 * sc,
            2.2 * sc,
            rgba(255, 230, 140, tw),
        );
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
        assert!(t5.crown && t5.collar && t5.bow && t5.jewels && t5.cape);
        assert!(!t1.cape && !t1.crown);
        assert!(t5.fluff > t1.fluff + 0.5);
        assert!(t5.scale > t1.scale + 0.2);
        let t4 = look_for("มะลิ", 3);
        assert!(t4.jewels && !t4.crown && !t4.cape);
        assert!(t4.fluff > look_for("มะลิ", 2).fluff);
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

    #[test]
    fn luxury_escalation_durations_and_looks() {
        assert!(eat_dur(4) >= eat_dur(0) + 1.4);
        assert!(pour_dur(4) > pour_dur(0));
        assert!(reveal_dur(4, 5.0) >= reveal_dur(0, 5.0) + 2.0);
        assert!(walk_max(4) > walk_max(0));
        assert!(treat_for(4) == Treat::Feast);
        let t3 = look_for("สมชาย", 2);
        let t5 = look_for("จักรพรรดิเพชร", 4);
        assert!(t3.bow && t3.jewels == false);
        // jewels start at tier index 3 (display tier 4)
        assert!(look_for("สมชาย", 3).jewels);
        assert!(t5.cape && t5.crown);
        // high tiers last long enough that a mid-eat still is well after the walk-in
        assert!(walk_max(4) + pour_dur(4) + eat_dur(4) * 0.4 > 4.0);
        // hero cat is 1.5× / 2× tier 1; queued high-tier cats stay compact
        assert!((hero_body_scale(3) / hero_body_scale(0) - 1.5).abs() < 0.01);
        assert!((hero_body_scale(4) / hero_body_scale(0) - 2.0).abs() < 0.01);
        assert!(queue_body_scale(4) < hero_body_scale(0) * 1.25);
        assert!(front_stand(4) < 540.0 && front_stand(4) > 400.0);
        assert!(treat_x(4) > front_stand(4) + 200.0);
        let xs = queue_targets(&["จักรพรรดิเพชร", "มะลิ", "สมชาย"], &[4usize, 0, 1], 5);
        assert!((xs[0] - front_stand(4)).abs() < 0.1);
        assert!(xs[1] < xs[0] - 160.0);
        assert!(xs[2] > 0.0, "queued cats must stay on-screen, xs={xs:?}");
        assert!(xs[1] - xs[2] >= 120.0, "waiters must not stack, xs={xs:?}");
    }
}
