//! โหมดปาตกน้ำ — ส่งของขวัญแล้วตัวละครปาลูกบอลใส่เป้า เต็มเกจหรือขั้น 5 = ดังก์!
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::VecDeque;
use std::f32::consts::PI;

pub const WALK: f32 = 1.35;
pub const LEAVE: f32 = 1.25;
const DEFAULT_BALLS: [u32; 5] = [1, 2, 4, 7, 3];
const DEFAULT_ADD: [u32; 5] = [1, 2, 4, 7, 10];
const DEFAULT_HIT: [f32; 5] = [0.45, 0.55, 0.72, 0.86, 1.0];
const THROW_EACH: f32 = 0.72;
const DROP: f32 = 0.55;
const SPLASH: f32 = 2.4;
const CELEBRATE: f32 = 2.8;

pub fn floor_y(vh: f32) -> f32 {
    vh * 0.86
}

pub struct DunkIn {
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub gift: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Walk,
    Throw,
    Drop,
    Splash,
    Leave,
    Done,
}

#[derive(Clone, Debug)]
pub struct Ball {
    pub t0: f32, // time in the throw phase when this ball is released
    pub hit: bool,
    pub size: f32,
}

#[derive(Clone, Debug)]
pub struct Show {
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub gift: String,
    pub t: f32,
    pub walk: f32,
    pub throw: f32,
    pub leave: f32,
    pub look: Look,
    pub seed: u64,
    pub x: f32,
    pub appear: f32,
    pub balls: Vec<Ball>,
    pub hits: u32,
    pub add: u32,
    pub will_dunk: bool,
}

impl Show {
    pub fn phase(&self) -> Phase {
        if self.t < self.walk {
            Phase::Walk
        } else if self.t < self.walk + self.throw {
            Phase::Throw
        } else if self.will_dunk && self.t < self.walk + self.throw + DROP {
            Phase::Drop
        } else if self.will_dunk && self.t < self.walk + self.throw + DROP + SPLASH {
            Phase::Splash
        } else if self.t < self.total() {
            Phase::Leave
        } else {
            Phase::Done
        }
    }
    pub fn phase_name(&self) -> &'static str {
        match self.phase() {
            Phase::Walk => "walk",
            Phase::Throw => "throw",
            Phase::Drop => "drop",
            Phase::Splash => "splash",
            Phase::Leave => "leave",
            Phase::Done => "done",
        }
    }
    pub fn total(&self) -> f32 {
        if self.will_dunk {
            self.walk + self.throw + DROP + SPLASH + CELEBRATE
        } else {
            self.walk + self.throw + self.leave
        }
    }
    pub fn is_done(&self) -> bool {
        self.t >= self.total()
    }
}

struct Dropper {
    look: Look,
    y_off: f32, // 0 = on seat, 1 = in water
    wet: f32,
}

struct SplashBit {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    r: f32,
}

pub struct Stage {
    rng: fastrand::Rng,
    shows: VecDeque<Show>,
    waiting: VecDeque<DunkIn>,
    meter: u32,
    target: u32,
    last_dunk_by: Option<String>,
    last_dunk_t: f32,
    dropper: Dropper,
    pub started: u64,
    pub finished: u64,
    bits: Vec<SplashBit>,
    clock: f32,
    /// how long the DUNK! banner has been flashing (0 = hidden)
    banner: f32,
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage {
    pub fn new() -> Self {
        Self {
            rng: fastrand::Rng::new(),
            shows: VecDeque::new(),
            waiting: VecDeque::new(),
            meter: 0,
            target: 10,
            last_dunk_by: None,
            last_dunk_t: 99.0,
            dropper: Dropper { look: host_look(), y_off: 0.0, wet: 0.0 },
            started: 0,
            finished: 0,
            bits: vec![],
            clock: 0.0,
            banner: 0.0,
        }
    }
    pub fn with_seed(seed: u64) -> Self {
        let mut s = Self::new();
        s.rng = fastrand::Rng::with_seed(seed);
        s
    }
    pub fn clear(&mut self) {
        self.shows.clear();
        self.waiting.clear();
        self.bits.clear();
        self.meter = 0;
        self.banner = 0.0;
        self.dropper.y_off = 0.0;
        self.dropper.wet = 0.0;
        self.last_dunk_by = None;
        self.started = 0;
        self.finished = 0;
    }
    pub fn busy(&self) -> bool {
        !self.shows.is_empty() || !self.waiting.is_empty() || self.banner > 0.0
    }
    pub fn len(&self) -> usize {
        self.shows.len() + self.waiting.len()
    }
    pub fn front_user(&self) -> Option<&str> {
        self.shows.front().map(|s| s.user.as_str())
    }
    pub fn lead(&self) -> Option<&Show> {
        self.shows.front()
    }
    pub fn shows(&self) -> impl Iterator<Item = &Show> {
        self.shows.iter()
    }
    pub fn waiting_names(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.shows.iter().skip(1).map(|s| s.user.as_str()).collect();
        v.extend(self.waiting.iter().map(|w| w.user.as_str()));
        v
    }
    pub fn meter(&self) -> u32 {
        self.meter
    }
    pub fn target(&self) -> u32 {
        self.target
    }
    pub fn last_dunk_by(&self) -> Option<&str> {
        self.last_dunk_by.as_deref()
    }

    pub fn enqueue(
        &mut self,
        inn: DunkIn,
        meter_target: u32,
        meter_add: &[u32],
        balls: &[u32],
        instant_tier_1based: usize,
        max_visible: usize,
    ) {
        self.target = meter_target.max(1);
        let cap = max_visible.max(1).min(8);
        if self.shows.is_empty() {
            let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
            self.started += 1;
            self.shows.push_back(show);
        } else if self.shows.len() < cap {
            // backups wait on the side; they are already "on stage" for name tags
            let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
            self.started += 1;
            self.shows.push_back(show);
        } else {
            self.waiting.push_back(inn);
        }
    }

    fn make_show(&mut self, inn: DunkIn, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize) -> Show {
        let tier = inn.tier.min(4);
        let n = balls.get(tier).copied().unwrap_or(DEFAULT_BALLS[tier]).max(1);
        let add = meter_add.get(tier).copied().unwrap_or(DEFAULT_ADD[tier]).max(1);
        let chance = DEFAULT_HIT[tier];
        let size = [10.0, 13.0, 17.0, 22.0, 28.0][tier];
        let mut list = Vec::with_capacity(n as usize);
        let mut hits = 0u32;
        for i in 0..n {
            let hit = if tier == 4 { true } else { self.rng.f32() < chance };
            if hit {
                hits += 1;
            }
            list.push(Ball { t0: 0.18 + i as f32 * THROW_EACH, hit, size });
        }
        let throw = 0.18 + n as f32 * THROW_EACH + 0.2;
        // will_dunk is resolved after the throw (we apply meter then); precompute a tentative flag
        // actual flag is set at the end of throw in tick, but we need duration now.
        // Use a conservative "might dunk" and fix duration when throw ends.
        let seed = self.rng.u64(..);
        let _ = instant_tier_1based;
        Show {
            look: appearance_for(&inn.user, inn.tier),
            user: inn.user,
            tier: inn.tier,
            tier_name: inn.tier_name,
            gift: inn.gift,
            t: 0.0,
            walk: WALK,
            throw,
            leave: LEAVE,
            seed,
            x: -80.0,
            appear: 0.0,
            balls: list,
            hits,
            add,
            will_dunk: false, // set when throw completes
        }
    }

    pub fn tick(&mut self, dt: f32, meter_target: u32, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize, max_visible: usize) {
        self.clock += dt;
        self.target = meter_target.max(1);
        let cap = max_visible.max(1).min(8);

        // only the lead throws; backups hold after a short walk-in
        if let Some(lead) = self.shows.front_mut() {
            let prev = lead.t;
            lead.t += dt;
            lead.appear = (lead.appear + dt / 0.3).min(1.0);
            // resolve dunk at the instant the last ball finishes
            if prev < lead.walk + lead.throw && lead.t >= lead.walk + lead.throw && !lead.will_dunk {
                let instant = inn_is_instant(lead.tier, instant_tier_1based);
                let gained = if lead.hits > 0 { lead.add } else { 0 };
                self.meter = self.meter.saturating_add(gained);
                if instant || self.meter >= self.target {
                    lead.will_dunk = true;
                    self.meter = 0;
                    self.last_dunk_by = Some(lead.user.clone());
                    self.last_dunk_t = 0.0;
                    self.banner = 2.6;
                }
            }
        }
        for (i, s) in self.shows.iter_mut().enumerate() {
            if i == 0 {
                continue;
            }
            if s.t < s.walk {
                s.t = (s.t + dt).min(s.walk);
            }
            s.appear = (s.appear + dt / 0.3).min(1.0);
        }

        // animate host
        let phase = self.shows.front().map(|s| s.phase());
        match phase {
            Some(Phase::Drop) => {
                let u = if let Some(s) = self.shows.front() {
                    clamp01((s.t - s.walk - s.throw) / DROP)
                } else {
                    1.0
                };
                self.dropper.y_off = ease_in_out(u);
            }
            Some(Phase::Splash) => {
                self.dropper.y_off = 1.0;
                self.dropper.wet = 1.0;
                if self.rng.f32() < 40.0 * dt {
                    self.bits.push(SplashBit {
                        x: tank_x() + self.rng.f32() * 160.0 - 80.0,
                        y: tank_water_y(floor_y(720.0)),
                        vx: self.rng.f32() * 240.0 - 120.0,
                        vy: -220.0 - self.rng.f32() * 180.0,
                        life: 0.8,
                        max: 0.8,
                        r: 6.0 + self.rng.f32() * 10.0,
                    });
                }
            }
            Some(Phase::Leave) if self.shows.front().map(|s| s.will_dunk).unwrap_or(false) => {
                // climb back onto the seat after splash
                if let Some(s) = self.shows.front() {
                    let u = clamp01((s.t - s.walk - s.throw - DROP - SPLASH) / 0.8);
                    self.dropper.y_off = 1.0 - ease_in_out(u);
                    self.dropper.wet = 1.0 - u * 0.4;
                }
            }
            _ => {
                if self.shows.front().is_none() {
                    self.dropper.y_off = (self.dropper.y_off - dt * 1.2).max(0.0);
                    self.dropper.wet = (self.dropper.wet - dt * 0.3).max(0.0);
                }
            }
        }
        if self.banner > 0.0 {
            self.banner -= dt;
        }
        self.last_dunk_t += dt;

        for b in &mut self.bits {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.vy += 520.0 * dt;
            b.life -= dt;
        }
        self.bits.retain(|b| b.life > 0.0);

        if self.shows.front().map(|s| s.is_done()).unwrap_or(false) {
            self.finished += 1;
            self.shows.pop_front();
            if let Some(next) = self.shows.front_mut() {
                if next.t >= next.walk {
                    next.t = 0.0; // walk up to the throw line again? start throw from current side
                    next.t = next.walk * 0.6;
                }
            }
        }
        while self.shows.len() < cap {
            let Some(inn) = self.waiting.pop_front() else { break };
            let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
            self.started += 1;
            self.shows.push_back(show);
        }
    }

    pub fn draw_world(&self, cx: &Cx, pcx: &Cx, sprites: &Sprites, bg: &str, max_visible: usize) {
        let g = floor_y(cx.vh);
        if bg == "scene" {
            draw_dunk_sky(cx, g);
        }
        let set = if bg == "scene" { cx } else { pcx };
        draw_tank(set, g, self.meter, self.target, self.dropper.y_off);
        // host on the seat (or falling)
        self.draw_host(cx, sprites, g);
        // throwers: backups first
        let n = self.shows.len().max(1);
        for i in (1..self.shows.len()).rev() {
            self.draw_thrower(cx, sprites, i, n, g, false);
        }
        if !self.shows.is_empty() {
            self.draw_thrower(cx, sprites, 0, n, g, true);
        }
        self.draw_balls(cx, g);
        for b in &self.bits {
            let a = clamp01(b.life / b.max);
            cx.ellipse(b.x, b.y, b.r, b.r * 0.7, rgba(200, 230, 255, 0.7 * a));
        }
        let _ = max_visible;
    }

    fn draw_host(&self, cx: &Cx, sprites: &Sprites, g: f32) {
        let seat_y = seat_ground(g);
        let water_y = tank_water_y(g) + 20.0;
        let gy = lerp(seat_y, water_y, self.dropper.y_off);
        let mut hcx = cx.with_painter(cx.p);
        hcx.gy = Some(gy);
        let mut pose = Pose::default();
        if self.dropper.y_off < 0.05 {
            pose.expr = 2;
            pose.breath = (self.clock * 2.0).sin() * 0.4;
        } else if self.dropper.y_off < 1.0 {
            pose.expr = 0;
            pose.squash = -0.25;
            pose.hands = 0.8;
            pose.raise = 0.9;
        } else {
            pose.expr = 0;
            pose.hands = 0.5;
            pose.squash = 0.2;
            pose.blink = 0.4;
        }
        let mut look = self.dropper.look;
        look.scale = 0.95;
        let rig = draw_character(&hcx, sprites, tank_x(), &pose, &look);
        let tag_y = rig.head.1 - HEAD_R * look.scale - 16.0;
        draw_name_banner(cx, tank_x(), tag_y, "ผู้ไลฟ์", 0, "", 1.0, 1.0);
        let _ = self.dropper.wet;
    }

    fn draw_thrower(&self, cx: &Cx, sprites: &Sprites, i: usize, n: usize, g: f32, is_lead: bool) {
        let s = &self.shows[i];
        let target_x = if is_lead { 380.0 } else { 90.0 + (i as f32 - 1.0) * 120.0 };
        let walk_u = clamp01(s.t / s.walk);
        let leave_u = if matches!(s.phase(), Phase::Leave | Phase::Done) && !s.will_dunk {
            clamp01((s.t - s.walk - s.throw) / s.leave)
        } else if s.will_dunk && matches!(s.phase(), Phase::Leave | Phase::Done) {
            clamp01((s.t - s.walk - s.throw - DROP - SPLASH) / CELEBRATE)
        } else {
            0.0
        };
        let x = if s.t < s.walk {
            lerp(-90.0, target_x, ease_in_out(walk_u))
        } else if !s.will_dunk && leave_u > 0.0 {
            lerp(target_x, -90.0, ease_in_out(leave_u))
        } else {
            target_x
        };
        let mut pose = Pose::default();
        pose.facing = 1.0;
        if s.t < s.walk {
            pose.walk = 1.0;
            pose.phase = walk_u * 8.0;
        } else if s.phase() == Phase::Throw {
            let tt = s.t - s.walk;
            // find current ball
            let mut wind = 0.0;
            for b in &s.balls {
                let u = tt - b.t0 + 0.35;
                if u >= 0.0 && u < 0.5 {
                    wind = 1.0 - (u / 0.5 - 0.15).abs().min(1.0);
                }
            }
            pose.hands = 0.9;
            pose.raise = 0.3 + 0.6 * wind;
            pose.squash = -0.15 * wind;
            pose.expr = 2;
            pose.walk = 0.0;
        } else if s.will_dunk && matches!(s.phase(), Phase::Drop | Phase::Splash | Phase::Leave) {
            pose.expr = 1;
            pose.hands = 0.8;
            pose.raise = 0.85;
            pose.breath = (self.clock * 6.0).sin();
        } else if leave_u > 0.0 {
            pose.walk = 1.0;
            pose.phase = leave_u * 7.0;
            pose.facing = -1.0;
        } else {
            pose.breath = (self.clock * 2.0 + i as f32).sin() * 0.4;
        }
        let mut look = s.look;
        look.scale = if is_lead { 1.05 } else { 0.86 };
        let mut tcx = cx.with_painter(cx.p);
        tcx.gy = Some(g);
        let rig = draw_character(&tcx, sprites, x, &pose, &look);
        let tag_y = rig.head.1 - HEAD_R * look.scale - 18.0;
        let alpha = if leave_u > 0.6 && !s.will_dunk { 1.0 - (leave_u - 0.6) / 0.4 } else { s.appear };
        draw_name_banner(cx, x, tag_y, &s.user, s.tier, if is_lead { &s.tier_name } else { "" }, s.appear, alpha);
    }

    fn draw_balls(&self, cx: &Cx, g: f32) {
        let Some(s) = self.shows.front() else { return };
        if s.t < s.walk {
            return;
        }
        let tt = s.t - s.walk;
        let origin = (380.0 + 30.0, g - 110.0);
        let target = (tank_x() - 90.0, g - 210.0); // the dunk target pad
        for b in &s.balls {
            let u = (tt - b.t0) / 0.42;
            if u < 0.0 || u > 1.15 {
                continue;
            }
            let fly = clamp01(u);
            let mut end = if b.hit {
                target
            } else {
                (target.0 + 80.0, target.1 - 40.0 + 180.0 * (fly * fly))
            };
            if !b.hit && fly > 0.7 {
                end.1 += (fly - 0.7) * 200.0;
            }
            let x = lerp(origin.0, end.0, ease_in_out(fly.min(1.0)));
            let arc = -140.0 * (4.0 * fly * (1.0 - fly));
            let y = lerp(origin.1, end.1, fly.min(1.0)) + arc;
            if fly > 1.0 {
                continue;
            }
            cx.circle(x, y, b.size, rgba(255, 90, 70, 1.0));
            cx.circle(x - b.size * 0.3, y - b.size * 0.3, b.size * 0.28, rgba(255, 200, 180, 0.8));
        }
    }

    pub fn draw_overlay(&self, cx: &Cx, card_title: &str, dunk_banner: &str, splash_text: &str) {
        // meter (always when busy or meter > 0)
        if self.busy() || self.meter > 0 {
            draw_meter(cx, self.meter, self.target);
        }
        let Some(s) = self.shows.front() else {
            return;
        };
        // flashing DUNK! banner — cue for a real-life operator
        if self.banner > 0.0 && s.will_dunk && matches!(s.phase(), Phase::Drop | Phase::Splash | Phase::Leave) {
            let flash = 0.65 + 0.35 * (cx.time * 10.0).sin().abs();
            let a = clamp01(self.banner / 0.3).min(clamp01(self.banner)) * flash;
            let text = if dunk_banner.is_empty() { "ดังก์! DUNK!" } else { dunk_banner };
            cx.rect(140.0, 200.0, VW - 140.0, 340.0, 24.0, rgba(180, 10, 40, 0.72 * a));
            cx.rect_line(140.0, 200.0, VW - 140.0, 340.0, 24.0, 6.0, rgba(255, 230, 80, 0.9 * a));
            cx.text(VW * 0.5, 270.0, Align2::CENTER_CENTER, text, 72.0, rgba(255, 255, 255, a), true);
        }
        // splash credit card
        if s.will_dunk && matches!(s.phase(), Phase::Splash | Phase::Leave) {
            let fade = if s.phase() == Phase::Leave {
                1.0 - clamp01((s.t - s.walk - s.throw - DROP - SPLASH) / 0.6)
            } else {
                clamp01((s.t - s.walk - s.throw - DROP) / 0.25)
            };
            if fade > 0.02 {
                let msg = splash_text.replace("{name}", &s.user);
                let msg = if splash_text.is_empty() {
                    format!("SPLASH! ตกน้ำเพราะ {}", s.user)
                } else {
                    msg
                };
                let title = if card_title.is_empty() { "ตกน้ำ!" } else { card_title };
                cx.rect(280.0, 40.0, VW - 280.0, 160.0, 18.0, rgba(8, 20, 50, 0.82 * fade));
                cx.rect_line(280.0, 40.0, VW - 280.0, 160.0, 18.0, 3.0, rgba(80, 180, 255, 0.8 * fade));
                cx.text(VW * 0.5, 70.0, Align2::CENTER_CENTER, title, 22.0, rgba(180, 220, 255, fade), true);
                cx.text(VW * 0.5, 112.0, Align2::CENTER_CENTER, &msg, 32.0, rgba(255, 255, 255, fade), true);
            }
        }
    }
}

fn inn_is_instant(tier_0: usize, instant_1based: usize) -> bool {
    let thr = if instant_1based == 0 { 5 } else { instant_1based };
    (tier_0 + 1) >= thr
}

fn tank_x() -> f32 {
    900.0
}
fn seat_ground(g: f32) -> f32 {
    g - 268.0
}
fn tank_water_y(g: f32) -> f32 {
    g - 90.0
}

fn draw_dunk_sky(cx: &Cx, g: f32) {
    cx.vgradient(-40.0, -40.0, VW + 40.0, cx.vh + 40.0, rgba(70, 150, 210, 1.0), rgba(180, 220, 170, 1.0));
    cx.glow(200.0, 80.0, 140.0, rgba(255, 240, 180, 1.0), 0.4);
    // carnival tents in the back
    for (x, w, col) in [(40.0, 200.0, rgba(200, 50, 60, 0.9)), (1080.0, 180.0, rgba(50, 90, 180, 0.9))] {
        cx.poly(&[(x, g - 40.0), (x + w * 0.5, g - 200.0), (x + w, g - 40.0)], col);
        cx.rect(x + 20.0, g - 40.0, x + w - 20.0, g, 0.0, rgba(240, 230, 210, 0.9));
    }
    cx.rect(-20.0, g, VW + 20.0, cx.vh + 40.0, 0.0, rgba(90, 160, 80, 1.0));
}

fn draw_tank(cx: &Cx, g: f32, meter: u32, target: u32, drop: f32) {
    let x = tank_x();
    let tank_top = g - 200.0;
    let tank_bot = g - 10.0;
    let tw = 150.0;
    // frame
    cx.rect(x - tw, tank_top, x + tw, tank_bot, 8.0, rgba(80, 140, 180, 0.35));
    cx.rect_line(x - tw, tank_top, x + tw, tank_bot, 8.0, 6.0, rgba(40, 80, 110, 1.0));
    // water
    let fill = 0.55 + 0.15 * (cx.time * 1.5).sin();
    let wy = lerp(tank_bot, tank_top + 16.0, fill);
    cx.rect(x - tw + 6.0, wy, x + tw - 6.0, tank_bot - 4.0, 4.0, rgba(60, 150, 210, 0.7));
    cx.ellipse(x, wy, tw - 8.0, 10.0, rgba(100, 190, 230, 0.7));
    // ripples when dunked
    if drop > 0.4 {
        for i in 0..3 {
            let r = 20.0 + drop * 40.0 + i as f32 * 18.0;
            cx.ellipse(x, wy, r, 8.0 + i as f32 * 3.0, rgba(255, 255, 255, 0.2 * (1.0 - drop * 0.3)));
        }
    }
    // support posts + seat
    cx.rect(x - tw - 16.0, tank_top - 80.0, x - tw - 4.0, tank_bot, 3.0, rgba(80, 70, 60, 1.0));
    cx.rect(x + tw + 4.0, tank_top - 80.0, x + tw + 16.0, tank_bot, 3.0, rgba(80, 70, 60, 1.0));
    let seat_y = tank_top - 68.0 + drop * 40.0; // seat tips a little
    let tilt = drop * 0.8;
    // seat plank
    cx.rect(x - 50.0, seat_y, x + 50.0, seat_y + 14.0, 4.0, rgba(160, 90, 40, 1.0));
    if drop > 0.15 {
        // hinged seat falling
        let ang = tilt.min(1.0) * 1.2;
        let (sn, cs) = ang.sin_cos();
        let p0 = (x - 50.0, seat_y);
        let p1 = (x - 50.0 + 100.0 * cs, seat_y + 100.0 * sn);
        cx.seg(p0, p1, 12.0, rgba(160, 90, 40, 1.0));
    }
    // target pad
    let tx = x - tw - 36.0;
    let ty = tank_top - 10.0;
    cx.circle(tx, ty, 34.0, rgba(240, 240, 240, 1.0));
    cx.circle(tx, ty, 24.0, rgba(220, 40, 40, 1.0));
    cx.circle(tx, ty, 12.0, rgba(240, 240, 240, 1.0));
    cx.circle(tx, ty, 5.0, rgba(220, 40, 40, 1.0));
    // latch
    cx.rect(tx + 20.0, ty - 4.0, x - tw, ty + 4.0, 2.0, rgba(60, 60, 70, 1.0));
    let _ = meter;
    let _ = target;
}

fn draw_meter(cx: &Cx, meter: u32, target: u32) {
    let x0 = 40.0;
    let y0 = 36.0;
    let w = 260.0;
    let h = 28.0;
    cx.rect(x0, y0, x0 + w, y0 + h, 8.0, rgba(10, 20, 40, 0.75));
    let u = (meter as f32 / target.max(1) as f32).clamp(0.0, 1.0);
    cx.rect(x0 + 3.0, y0 + 3.0, x0 + 3.0 + (w - 6.0) * u, y0 + h - 3.0, 6.0, rgba(80, 180, 255, 0.95));
    cx.rect_line(x0, y0, x0 + w, y0 + h, 8.0, 2.0, rgba(200, 230, 255, 0.8));
    cx.text(x0 + w * 0.5, y0 + h * 0.5, Align2::CENTER_CENTER, &format!("เกจดังก์  {meter}/{target}"), 18.0, rgba(255, 255, 255, 0.95), true);
}

fn host_look() -> Look {
    Look { tier: 2, shirt: Color32::from_rgb(80, 170, 230), pants: Color32::from_rgb(40, 50, 80), scale: 1.0, hair: Color32::from_rgb(50, 30, 20), style: 1 }
}

fn appearance_for(name: &str, tier: usize) -> Look {
    let mut h = 0xcbf29ce484222325u64;
    for b in name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    Look {
        tier,
        shirt: hsv(((h >> 1) % 360) as f32, 0.55, 0.92),
        pants: hsv(((h >> 9) % 360) as f32, 0.4, 0.38),
        scale: 1.0,
        hair: hsv(((h >> 17) % 360) as f32, 0.45, 0.28),
        style: ((h >> 24) % 6) as u8,
    }
}

fn hsv(h: f32, s: f32, v: f32) -> Color32 {
    let c = v * s;
    let h6 = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h6 % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h6 as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color32::from_rgb(((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inn(user: &str, tier: usize) -> DunkIn {
        DunkIn { user: user.into(), tier, tier_name: format!("ขั้น{}", tier + 1), gift: "Rose".into() }
    }
    const ADD: [u32; 5] = [1, 2, 4, 7, 10];
    const BALLS: [u32; 5] = [1, 2, 4, 7, 3];

    fn run_one(st: &mut Stage, inn: DunkIn, target: u32) {
        st.enqueue(inn, target, &ADD, &BALLS, 5, 6);
        // walk + throw + maybe splash
        for _ in 0..800 {
            if !st.busy() {
                break;
            }
            // if lead is done and no more, stop
            st.tick(0.05, target, &ADD, &BALLS, 5, 6);
            if st.shows.is_empty() && st.waiting.is_empty() {
                break;
            }
        }
    }

    #[test]
    fn t5_always_dunks() {
        let mut st = Stage::with_seed(7);
        st.enqueue(inn("นก", 4), 10, &ADD, &BALLS, 5, 6);
        // finish walk+throw
        let throw_end = WALK + st.lead().unwrap().throw + 0.05;
        st.tick(throw_end, 10, &ADD, &BALLS, 5, 6);
        let s = st.lead().unwrap();
        assert!(s.will_dunk, "t5 must dunk");
        assert_eq!(st.last_dunk_by(), Some("นก"));
        assert_eq!(st.meter(), 0, "meter resets after dunk");
        assert!(matches!(s.phase(), Phase::Drop | Phase::Splash));
    }

    #[test]
    fn meter_fills_then_dunks() {
        let mut st = Stage::with_seed(1);
        // target 2, t2 adds 2 if any hit — may miss. Use many t1 or force by looping.
        // Use target=1 and a mid tier with seed hunt, or just add until dunk.
        let mut dunked = false;
        for i in 0..12 {
            let name = format!("คน{i}");
            st.enqueue(inn(&name, 2), 10, &ADD, &BALLS, 5, 6);
            // run this throw only
            if let Some(lead) = st.lead() {
                let end = lead.walk + lead.throw + 0.05;
                let dt = end - lead.t;
                if dt > 0.0 {
                    st.tick(dt, 10, &ADD, &BALLS, 5, 6);
                }
            }
            if st.last_dunk_by().is_some() {
                dunked = true;
                assert_eq!(st.meter(), 0);
                break;
            }
            // finish leave so next starts
            if let Some(lead) = st.lead() {
                let rest = lead.total() - lead.t + 0.05;
                if rest > 0.0 {
                    st.tick(rest, 10, &ADD, &BALLS, 5, 6);
                }
            }
        }
        assert!(dunked, "meter 10 should dunk after several t3 throws");
    }

    #[test]
    fn queue_of_three() {
        let mut st = Stage::with_seed(3);
        st.enqueue(inn("เอ", 0), 10, &ADD, &BALLS, 5, 6);
        st.enqueue(inn("บี", 1), 10, &ADD, &BALLS, 5, 6);
        st.enqueue(inn("ซี", 2), 10, &ADD, &BALLS, 5, 6);
        assert_eq!(st.len(), 3);
        assert_eq!(st.front_user(), Some("เอ"));
        assert_eq!(st.waiting_names(), vec!["บี", "ซี"]);
    }

    #[test]
    fn instant_tier_config() {
        assert!(inn_is_instant(4, 5));
        assert!(inn_is_instant(4, 4));
        assert!(!inn_is_instant(2, 5));
        assert!(inn_is_instant(2, 3));
    }

    #[test]
    fn splash_credits_name() {
        let mut st = Stage::with_seed(9);
        st.enqueue(inn("สมศรี", 4), 10, &ADD, &BALLS, 5, 6);
        let end = WALK + st.lead().unwrap().throw + DROP + 0.3;
        st.tick(end, 10, &ADD, &BALLS, 5, 6);
        assert_eq!(st.last_dunk_by(), Some("สมศรี"));
        assert_eq!(st.lead().unwrap().phase(), Phase::Splash);
    }

    #[test]
    fn configurable_target() {
        let mut st = Stage::with_seed(4);
        // t5 fills/instant regardless of target
        st.enqueue(inn("เอ", 4), 99, &ADD, &BALLS, 5, 6);
        st.tick(WALK + st.lead().unwrap().throw + 0.05, 99, &ADD, &BALLS, 5, 6);
        assert!(st.lead().unwrap().will_dunk);
    }
}
