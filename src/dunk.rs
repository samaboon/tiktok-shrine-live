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
const THROW_EACH: f32 = 0.85;
const FLY: f32 = 0.70;
const LEAD_X: f32 = 580.0;
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
    pub points: u32,
    pub scored: bool,
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
    pub combo: u32,
    last_gift_at: f32,
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
    waiting: VecDeque<Pending>,
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
    /// smoothed meter for fill animation
    meter_disp: f32,
    /// "โดน!" pops
    pops: Vec<Pop>,
    flash: f32,
    overflow: VecDeque<String>,
    overflow_extra: u32,
    recent: VecDeque<(f32, String)>,
    party: Option<DunkParty>,
    merge_window: f32,
    max_seconds: f32,
    party_threshold: u32,
}

struct DunkParty {
    remain: f32,
    names: VecDeque<String>,
    gifts: u32,
}

struct Pending {
    inn: DunkIn,
    combo: u32,
    last_gift_at: f32,
}

const MAX_RECENT: usize = 64;
const MAX_TICKER: usize = 48;
const MAX_WAITING: usize = 24;
const MAX_OVERFLOW_NAMES: usize = 64;
const MAX_BALLS: usize = 16;

struct Pop {
    x: f32,
    y: f32,
    life: f32,
    max: f32,
    text: &'static str,
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
            meter_disp: 0.0,
            pops: vec![],
            flash: 0.0,
            overflow: VecDeque::new(),
            overflow_extra: 0,
            recent: VecDeque::new(),
            party: None,
            merge_window: 4.0,
            max_seconds: 20.0,
            party_threshold: 8,
        }
    }
    pub fn set_flood(&mut self, merge_window: f32, max_seconds: f32, party_threshold: u32) {
        self.merge_window = merge_window.clamp(0.4, 30.0);
        self.max_seconds = max_seconds.clamp(4.0, 90.0);
        self.party_threshold = party_threshold.clamp(2, 99);
    }
    pub fn in_party(&self) -> bool {
        self.party.is_some()
    }
    pub fn combo(&self) -> u32 {
        self.shows.front().map(|s| s.combo).unwrap_or(1)
    }
    pub fn waiting_extra(&self) -> usize {
        self.waiting.len() + self.overflow.len() + self.overflow_extra as usize
    }
    pub fn live_units(&self) -> usize {
        self.shows.len()
            + self.waiting.len()
            + self.overflow.len()
            + self.recent.len()
            + self.bits.len()
            + self.pops.len()
            + self.party.as_ref().map(|p| p.names.len()).unwrap_or(0)
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
        self.meter_disp = 0.0;
        self.banner = 0.0;
        self.dropper.y_off = 0.0;
        self.dropper.wet = 0.0;
        self.last_dunk_by = None;
        self.started = 0;
        self.finished = 0;
        self.pops.clear();
        self.flash = 0.0;
        self.overflow.clear();
        self.overflow_extra = 0;
        self.recent.clear();
        self.party = None;
    }
    pub fn busy(&self) -> bool {
        !self.shows.is_empty() || !self.waiting.is_empty() || !self.overflow.is_empty() || self.overflow_extra > 0 || self.banner > 0.0 || self.party.is_some()
    }
    pub fn len(&self) -> usize {
        self.shows.len() + self.waiting_extra()
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
        v.extend(self.waiting.iter().map(|w| w.inn.user.as_str()));
        v.extend(self.overflow.iter().map(|s| s.as_str()));
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
        let cap = max_visible.max(1).min(12);
        self.note_gift(&inn.user);
        let tier = inn.tier.min(4);
        let add = meter_add.get(tier).copied().unwrap_or(DEFAULT_ADD[tier]).max(1);

        if self.try_merge(&inn, meter_add, balls, instant_tier_1based) {
            if self.recent_count() >= self.party_threshold as usize && self.party.is_none() {
                self.enter_party(&inn);
            }
            return;
        }
        if self.party.is_some() || self.recent_count() >= self.party_threshold as usize {
            self.enter_party(&inn);
            // gifts in party fill the meter immediately and share the ticker
            self.meter = self.meter.saturating_add(add);
            self.place_party_thrower(inn, meter_add, balls, instant_tier_1based, cap);
            return;
        }
        if self.shows.len() < cap {
            let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
            self.started += 1;
            self.shows.push_back(show);
        } else {
            self.push_waiting(Pending { inn, combo: 1, last_gift_at: self.clock });
        }
    }

    fn note_gift(&mut self, user: &str) {
        self.recent.push_back((self.clock, user.to_string()));
        let cut = self.clock - self.merge_window.max(1.0);
        while self.recent.front().map(|(t, _)| *t < cut).unwrap_or(false) {
            self.recent.pop_front();
        }
        while self.recent.len() > MAX_RECENT {
            self.recent.pop_front();
        }
    }

    fn recent_count(&self) -> usize {
        let cut = self.clock - self.merge_window.max(1.0);
        self.recent.iter().filter(|(t, _)| *t >= cut).count()
    }

    fn ticker_push(names: &mut VecDeque<String>, user: &str) {
        if names.back().map(|s| s == user).unwrap_or(false) {
            return;
        }
        names.push_back(user.to_string());
        while names.len() > MAX_TICKER {
            names.pop_front();
        }
    }

    fn enter_party(&mut self, inn: &DunkIn) {
        let extend = 3.0;
        if let Some(p) = &mut self.party {
            p.gifts = p.gifts.saturating_add(1);
            p.remain = (p.remain + extend).min(self.max_seconds);
            Self::ticker_push(&mut p.names, &inn.user);
            return;
        }
        let mut names = VecDeque::new();
        for s in &self.shows {
            Self::ticker_push(&mut names, &s.user);
        }
        for w in &self.waiting {
            Self::ticker_push(&mut names, &w.inn.user);
        }
        for o in &self.overflow {
            Self::ticker_push(&mut names, o);
        }
        Self::ticker_push(&mut names, &inn.user);
        let remain = (12.0 + self.recent_count() as f32 * 0.3).clamp(8.0, self.max_seconds);
        self.party = Some(DunkParty { remain, names, gifts: self.recent_count().max(1) as u32 });
    }

    fn place_party_thrower(&mut self, inn: DunkIn, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize, cap: usize) {
        if self.shows.iter().any(|s| s.user == inn.user) || self.waiting.iter().any(|w| w.inn.user == inn.user) {
            return;
        }
        if self.shows.len() < cap {
            let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
            self.started += 1;
            self.shows.push_back(show);
        } else {
            self.push_waiting(Pending { inn, combo: 1, last_gift_at: self.clock });
        }
    }

    fn push_waiting(&mut self, pend: Pending) {
        if self.waiting.len() < MAX_WAITING {
            self.waiting.push_back(pend);
        } else if self.overflow.len() < MAX_OVERFLOW_NAMES {
            self.overflow.push_back(pend.inn.user);
        } else {
            self.overflow_extra = self.overflow_extra.saturating_add(1);
        }
    }

    fn try_merge(&mut self, inn: &DunkIn, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize) -> bool {
        let window = self.merge_window;
        let clock = self.clock;
        let cap_s = self.max_seconds;
        let tier = inn.tier.min(4);
        if let Some(idx) = self.shows.iter().position(|s| s.user == inn.user && clock - s.last_gift_at <= window) {
            let is_lead = idx == 0;
            {
                let s = &mut self.shows[idx];
                s.combo = s.combo.saturating_add(1);
                s.last_gift_at = clock;
                if inn.tier > s.tier {
                    s.tier = inn.tier;
                    s.tier_name = inn.tier_name.clone();
                    s.look = appearance_for(&s.user, s.tier);
                }
            }
            if is_lead {
                self.append_balls(meter_add, balls, tier, cap_s);
            }
            let _ = instant_tier_1based;
            return true;
        }
        if let Some(w) = self.waiting.iter_mut().find(|w| w.inn.user == inn.user && clock - w.last_gift_at <= window) {
            w.combo = w.combo.saturating_add(1);
            w.last_gift_at = clock;
            if inn.tier > w.inn.tier {
                w.inn.tier = inn.tier;
                w.inn.tier_name = inn.tier_name.clone();
            }
            return true;
        }
        false
    }

    fn append_balls(&mut self, meter_add: &[u32], balls: &[u32], tier: usize, cap_s: f32) {
        let Some(lead) = self.shows.front_mut() else { return };
        if lead.balls.len() >= MAX_BALLS {
            // still credit the meter share so merged gifts aren't dropped
            let add = meter_add.get(tier).copied().unwrap_or(DEFAULT_ADD[tier]).max(1);
            lead.add = lead.add.saturating_add(add);
            return;
        }
        let n = balls.get(tier).copied().unwrap_or(DEFAULT_BALLS[tier]).max(1);
        let add = meter_add.get(tier).copied().unwrap_or(DEFAULT_ADD[tier]).max(1);
        let size = [30.0, 34.0, 38.0, 44.0, 52.0][tier.min(4)];
        let chance = DEFAULT_HIT[tier.min(4)];
        let tt = (lead.t - lead.walk).max(0.0);
        let start = tt + 0.12;
        let room = (MAX_BALLS - lead.balls.len()).min(n as usize);
        let base = add / n.max(1);
        let rem = add % n.max(1);
        for i in 0..room {
            let hit = if tier >= 4 { true } else { self.rng.f32() < chance };
            if hit {
                lead.hits += 1;
            }
            let points = base + if i < rem as usize { 1 } else { 0 };
            lead.balls.push(Ball { t0: start + i as f32 * THROW_EACH, hit, size, points, scored: false });
        }
        lead.add = lead.add.saturating_add(add);
        let last_t0 = lead.balls.last().map(|b| b.t0).unwrap_or(lead.throw);
        lead.throw = (last_t0 + FLY + 0.2).max(lead.throw).min(cap_s.max(lead.throw));
    }

    fn end_party(&mut self) {
        let extra = self.waiting_extra() as u64;
        self.waiting.clear();
        self.overflow.clear();
        self.overflow_extra = 0;
        self.finished += extra;
        self.party = None;
    }

    fn make_show(&mut self, inn: DunkIn, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize) -> Show {
        let tier = inn.tier.min(4);
        let n = balls.get(tier).copied().unwrap_or(DEFAULT_BALLS[tier]).max(1);
        let add = meter_add.get(tier).copied().unwrap_or(DEFAULT_ADD[tier]).max(1);
        let chance = DEFAULT_HIT[tier];
        let size = [30.0, 34.0, 38.0, 44.0, 52.0][tier];
        let mut list = Vec::with_capacity(n as usize);
        let mut hits = 0u32;
        let base = add / n.max(1);
        let rem = add % n.max(1);
        for i in 0..n {
            let hit = if tier == 4 { true } else { self.rng.f32() < chance };
            if hit {
                hits += 1;
            }
            let points = base + if i < rem { 1 } else { 0 };
            list.push(Ball { t0: 0.18 + i as f32 * THROW_EACH, hit, size, points, scored: false });
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
            combo: 1,
            last_gift_at: self.clock,
        }
    }

    pub fn tick(&mut self, dt: f32, meter_target: u32, meter_add: &[u32], balls: &[u32], instant_tier_1based: usize, max_visible: usize) {
        self.clock += dt;
        self.target = meter_target.max(1);
        let cap = max_visible.max(1).min(12);
        let party_ended = if let Some(p) = &mut self.party {
            p.remain -= dt;
            p.remain <= 0.0
        } else {
            false
        };
        if party_ended {
            self.end_party();
        }

        // only the lead throws; backups hold after a short walk-in
        if let Some(lead) = self.shows.front_mut() {
            let prev = lead.t;
            lead.t += dt;
            lead.appear = (lead.appear + dt / 0.3).min(1.0);
            // score each ball as it lands so the meter fills per hit
            if lead.t >= lead.walk {
                let tt = lead.t - lead.walk;
                let pad = (tank_x() - 186.0, floor_y(720.0) - 210.0);
                for b in &mut lead.balls {
                    if !b.scored && tt >= b.t0 + FLY {
                        b.scored = true;
                        if b.hit {
                            self.meter = self.meter.saturating_add(b.points);
                            self.flash = 0.32;
                            self.pops.push(Pop {
                                x: pad.0,
                                y: pad.1 - 10.0,
                                life: 0.75,
                                max: 0.75,
                                text: "โดน!",
                            });
                        }
                    }
                }
            }
            // resolve dunk after the throw phase (all balls have been seen)
            if prev < lead.walk + lead.throw && lead.t >= lead.walk + lead.throw && !lead.will_dunk {
                let instant = inn_is_instant(lead.tier, instant_tier_1based);
                if instant || self.meter >= self.target {
                    lead.will_dunk = true;
                    if instant {
                        self.meter = self.meter.max(self.target);
                    }
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
                if self.rng.f32() < 90.0 * dt {
                    self.bits.push(SplashBit {
                        x: tank_x() + self.rng.f32() * 200.0 - 100.0,
                        y: tank_water_y(floor_y(720.0)),
                        vx: self.rng.f32() * 320.0 - 160.0,
                        vy: -280.0 - self.rng.f32() * 240.0,
                        life: 1.0,
                        max: 1.0,
                        r: 14.0 + self.rng.f32() * 22.0,
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
        self.flash = (self.flash - dt).max(0.0);
        let k = (dt * 7.0).min(1.0);
        self.meter_disp += (self.meter as f32 - self.meter_disp) * k;
        for p in &mut self.pops {
            p.life -= dt;
            p.y -= 70.0 * dt;
        }
        self.pops.retain(|p| p.life > 0.0);

        for b in &mut self.bits {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.vy += 520.0 * dt;
            b.life -= dt;
        }
        self.bits.retain(|b| b.life > 0.0);
        while self.bits.len() > 160 {
            self.bits.remove(0);
        }
        while self.pops.len() > 24 {
            self.pops.remove(0);
        }

        if self.shows.front().map(|s| s.is_done()).unwrap_or(false) {
            let dunked = self.shows.front().map(|s| s.will_dunk).unwrap_or(false);
            self.finished += 1;
            self.shows.pop_front();
            if dunked {
                self.meter = 0;
            }
            if let Some(next) = self.shows.front_mut() {
                if next.t >= next.walk {
                    next.t = 0.0; // walk up to the throw line again? start throw from current side
                    next.t = next.walk * 0.6;
                }
            }
        }
        while self.shows.len() < cap {
            let Some(w) = self.waiting.pop_front() else {
                if let Some(user) = self.overflow.pop_front() {
                    let inn = DunkIn { user, tier: 0, tier_name: "ขั้น1".into(), gift: String::new() };
                    let show = self.make_show(inn, meter_add, balls, instant_tier_1based);
                    self.started += 1;
                    self.shows.push_back(show);
                    continue;
                }
                break;
            };
            let mut show = self.make_show(w.inn, meter_add, balls, instant_tier_1based);
            show.combo = w.combo.max(1);
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
        draw_tank(set, g, self.dropper.y_off, self.flash);
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
            cx.ellipse(b.x, b.y, b.r, b.r * 0.7, rgba(220, 245, 255, 0.85 * a));
            cx.ellipse(b.x, b.y - b.r * 0.15, b.r * 0.55, b.r * 0.4, rgba(255, 255, 255, 0.45 * a));
        }
        for pop in &self.pops {
            let a = clamp01(pop.life / pop.max);
            let sc = 0.85 + 0.4 * (1.0 - a);
            cx.text(pop.x, pop.y, Align2::CENTER_CENTER, pop.text, 42.0 * sc, rgba(255, 230, 80, a), true);
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
        // headphones — streamer look, distinct from gift senders
        let (hx, hy) = rig.head;
        let hr = HEAD_R * look.scale;
        let cup = rgba(25, 20, 30, 1.0);
        let accent = rgba(255, 80, 170, 1.0);
        cx.circle(hx - hr * 1.05, hy + 2.0, hr * 0.42, cup);
        cx.circle(hx + hr * 1.05, hy + 2.0, hr * 0.42, cup);
        cx.circle(hx - hr * 1.05, hy + 2.0, hr * 0.22, accent);
        cx.circle(hx + hr * 1.05, hy + 2.0, hr * 0.22, accent);
        cx.seg((hx - hr * 0.9, hy - hr * 0.55), (hx + hr * 0.9, hy - hr * 0.55), 6.0, cup);
        cx.seg((hx - hr * 0.9, hy - hr * 0.55), (hx - hr * 1.05, hy + 2.0), 5.0, cup);
        cx.seg((hx + hr * 0.9, hy - hr * 0.55), (hx + hr * 1.05, hy + 2.0), 5.0, cup);
        let _ = self.dropper.wet;
    }

    fn draw_thrower(&self, cx: &Cx, sprites: &Sprites, i: usize, n: usize, g: f32, is_lead: bool) {
        let s = &self.shows[i];
        let target_x = if is_lead {
            LEAD_X
        } else {
            let nw = (n as i32 - 1).max(1) as f32;
            let last = LEAD_X - 180.0;
            if nw <= 1.0 {
                200.0_f32.min(last)
            } else {
                200.0 + (i as f32 - 1.0) * (last - 200.0) / (nw - 1.0)
            }
        };
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

    fn ball_pos(origin: (f32, f32), target: (f32, f32), b: &Ball, fly: f32) -> (f32, f32) {
        let mut end = if b.hit {
            target
        } else {
            (target.0 + 90.0, target.1 - 30.0 + 180.0 * (fly * fly))
        };
        if !b.hit && fly > 0.7 {
            end.1 += (fly - 0.7) * 200.0;
        }
        let x = lerp(origin.0, end.0, ease_in_out(fly.min(1.0)));
        let arc = -160.0 * (4.0 * fly * (1.0 - fly));
        let y = lerp(origin.1, end.1, fly.min(1.0)) + arc;
        (x, y)
    }

    fn draw_balls(&self, cx: &Cx, g: f32) {
        let Some(s) = self.shows.front() else { return };
        if s.t < s.walk {
            return;
        }
        let tt = s.t - s.walk;
        let origin = (LEAD_X + 36.0, g - 118.0);
        let target = (tank_x() - 186.0, g - 210.0); // the dunk target pad
        for b in &s.balls {
            let u = (tt - b.t0) / FLY;
            if u < 0.0 || u > 1.05 {
                continue;
            }
            let fly = clamp01(u);
            if fly >= 1.0 {
                continue;
            }
            // motion trail
            for k in 1..=6 {
                let u2 = fly - k as f32 * 0.055;
                if u2 <= 0.0 {
                    break;
                }
                let (tx, ty) = Self::ball_pos(origin, target, b, u2);
                let fa = 0.5 * (1.0 - k as f32 / 6.0);
                let r = b.size * (1.0 - k as f32 * 0.1);
                cx.circle(tx, ty, r, rgba(255, 140, 70, fa));
            }
            let (x, y) = Self::ball_pos(origin, target, b, fly);
            cx.glow(x, y, b.size * 2.4, rgba(255, 160, 60, 1.0), 0.55);
            cx.circle(x, y, b.size, rgba(230, 40, 40, 1.0));
            cx.circle(x, y, b.size * 0.72, rgba(255, 90, 50, 1.0));
            cx.circle(x - b.size * 0.28, y - b.size * 0.32, b.size * 0.36, rgba(255, 245, 210, 1.0));
        }
    }

    pub fn draw_overlay(&self, cx: &Cx, card_title: &str, dunk_banner: &str, splash_text: &str) {
        // meter stays on the upper-left, clear of the host (tank at x=900)
        if self.busy() || self.meter > 0 || self.meter_disp > 0.05 {
            draw_meter(cx, self.meter_disp, self.target);
        }
        if let Some(p) = &self.party {
            cx.text(640.0, 36.0, Align2::CENTER_CENTER, "โหมดปาร์ตี้!", 32.0, rgba(255, 80, 200, 0.8 + 0.2 * (cx.time * 8.0).sin().abs()), true);
            let line = if p.names.is_empty() {
                String::new()
            } else {
                let joined = p.names.iter().cloned().collect::<Vec<_>>().join("   ·   ");
                format!("{joined}   ·   {joined}")
            };
            if !line.is_empty() {
                let y0 = cx.vh - 70.0;
                cx.rect(-20.0, y0, VW + 20.0, y0 + 56.0, 0.0, rgba(10, 0, 30, 0.82));
                let w = cx.measure(&line, 30.0, true).x.max(1.0);
                let scroll = (cx.time * 140.0) % w;
                cx.text(40.0 - scroll, y0 + 28.0, Align2::LEFT_CENTER, &line, 30.0, rgba(255, 230, 140, 1.0), true);
                cx.text(40.0 - scroll + w, y0 + 28.0, Align2::LEFT_CENTER, &line, 30.0, rgba(255, 230, 140, 1.0), true);
            }
        }
        let n = self.waiting_extra();
        if n > 0 {
            let txt = format!("+{n} คนกำลังรอ");
            let size = 30.0;
            let m = cx.measure(&txt, size, true);
            let x1 = 700.0;
            let y1 = 72.0;
            let x0 = x1 - m.x - 36.0;
            let y0 = 16.0;
            // keep clear of the meter (x=24..524, y=10..62) and the host
            cx.rect(x0, y0, x1, y1, 14.0, rgba(20, 8, 40, 0.85));
            cx.rect_line(x0, y0, x1, y1, 14.0, 3.0, rgba(255, 200, 80, 0.9));
            cx.text((x0 + x1) * 0.5, (y0 + y1) * 0.5, Align2::CENTER_CENTER, &txt, size, rgba(255, 255, 255, 1.0), true);
        }
        let Some(s) = self.shows.front() else {
            return;
        };
        if s.combo >= 2 {
            let badge = format!("×{}", s.combo);
            cx.rect(540.0, 10.0, 630.0, 62.0, 12.0, rgba(220, 40, 80, 0.95));
            cx.rect_line(540.0, 10.0, 630.0, 62.0, 12.0, 3.0, rgba(255, 230, 80, 0.95));
            cx.text(585.0, 36.0, Align2::CENTER_CENTER, &badge, 32.0, rgba(255, 255, 255, 1.0), true);
        }
        // banners live left-of-tank so they never cover the seated streamer
        let left = 36.0;
        let right = 720.0;
        // flashing DUNK! banner — cue for a real-life operator
        if self.banner > 0.0 && s.will_dunk && matches!(s.phase(), Phase::Drop | Phase::Splash | Phase::Leave) {
            let flash = 0.7 + 0.3 * (cx.time * 10.0).sin().abs();
            let a = clamp01(self.banner / 0.3).min(clamp01(self.banner)) * flash;
            let text = if dunk_banner.is_empty() { "ดังก์! DUNK!" } else { dunk_banner };
            cx.rect(left, 78.0, right, 168.0, 18.0, rgba(180, 10, 40, 0.82 * a));
            cx.rect_line(left, 78.0, right, 168.0, 18.0, 5.0, rgba(255, 230, 80, 0.95 * a));
            cx.text((left + right) * 0.5, 123.0, Align2::CENTER_CENTER, text, 56.0, rgba(255, 255, 255, a), true);
        }
        // splash credit card — stacked under the DUNK banner, still above the throwers
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
                cx.rect(left, 180.0, right, 310.0, 18.0, rgba(8, 20, 50, 0.88 * fade));
                cx.rect_line(left, 180.0, right, 310.0, 18.0, 3.0, rgba(80, 200, 255, 0.9 * fade));
                cx.text((left + right) * 0.5, 208.0, Align2::CENTER_CENTER, title, 26.0, rgba(180, 230, 255, fade), true);
                cx.text((left + right) * 0.5, 258.0, Align2::CENTER_CENTER, &msg, 34.0, rgba(255, 255, 255, fade), true);
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
    // dusk fairground sky
    cx.vgradient(-40.0, -40.0, VW + 40.0, g * 0.55, rgba(36, 24, 88, 1.0), rgba(255, 120, 90, 1.0));
    cx.vgradient(-40.0, g * 0.5, VW + 40.0, g + 20.0, rgba(255, 120, 90, 1.0), rgba(255, 190, 120, 1.0));
    cx.glow(VW * 0.78, g * 0.38, 240.0, rgba(255, 210, 130, 1.0), 0.55);
    // ferris wheel silhouette
    let (wx, wy, wr) = (1180.0, g - 310.0, 120.0);
    cx.circle(wx, wy, wr, rgba(30, 18, 50, 0.18));
    cx.seg((wx, wy), (wx, g - 8.0), 5.0, rgba(30, 18, 50, 0.7));
    cx.seg((wx - 28.0, g - 8.0), (wx + 28.0, g - 8.0), 6.0, rgba(30, 18, 50, 0.7));
    for i in 0..12 {
        let a = i as f32 / 12.0 * PI * 2.0 + cx.time * 0.12;
        let (s, c) = a.sin_cos();
        cx.seg((wx, wy), (wx + c * wr, wy + s * wr), 2.5, rgba(30, 18, 50, 0.55));
        cx.circle(wx + c * wr, wy + s * wr, 9.0, rgba(30, 18, 50, 0.7));
    }
    cx.circle(wx, wy, 14.0, rgba(30, 18, 50, 0.8));
    // right carnival tent, behind the tank
    {
        let (x, w) = (1088.0, 200.0);
        cx.poly(&[(x, g - 36.0), (x + w * 0.5, g - 230.0), (x + w, g - 36.0)], rgba(50, 70, 170, 0.95));
        cx.rect(x + 18.0, g - 36.0, x + w - 18.0, g, 0.0, rgba(240, 220, 190, 0.92));
        for i in 0..5 {
            let sx = x + 18.0 + i as f32 * ((w - 36.0) / 5.0);
            if i % 2 == 0 {
                cx.rect(sx, g - 36.0, sx + (w - 36.0) / 5.0, g, 0.0, rgba(200, 40, 70, 0.55));
            }
        }
    }
    // compact left game booth — stays in x≈0–130 so the queue is clear
    {
        let x0 = 4.0;
        let x1 = 126.0;
        cx.rect(x0, g - 130.0, x1, g, 0.0, rgba(70, 40, 30, 1.0));
        cx.rect(x0 + 6.0, g - 70.0, x1 - 6.0, g - 8.0, 4.0, rgba(160, 90, 40, 1.0));
        // striped awning
        cx.poly(&[(x0 - 8.0, g - 128.0), ((x0 + x1) * 0.5, g - 188.0), (x1 + 8.0, g - 128.0)], rgba(200, 40, 60, 1.0));
        cx.poly(&[(x0 - 2.0, g - 128.0), ((x0 + x1) * 0.5, g - 170.0), (x1 + 2.0, g - 128.0)], rgba(250, 230, 200, 0.95));
        for i in 0..6 {
            let u0 = i as f32 / 6.0;
            let u1 = (i + 1) as f32 / 6.0;
            if i % 2 == 0 {
                let l0 = (x0 - 8.0 + (x1 + 16.0) * u0, g - 128.0);
                let l1 = (x0 - 8.0 + (x1 + 16.0) * u1, g - 128.0);
                let peak = ((x0 + x1) * 0.5, g - 188.0);
                cx.poly(&[l0, peak, l1], rgba(200, 40, 60, 1.0));
            }
        }
        cx.rect(x0 + 14.0, g - 158.0, x1 - 14.0, g - 126.0, 6.0, rgba(40, 20, 20, 0.9));
        cx.text((x0 + x1) * 0.5, g - 142.0, Align2::CENTER_CENTER, "ปาเป้า", 18.0, rgba(255, 230, 120, 1.0), true);
    }
    // crowd silhouettes along the mid-ground
    for i in 0..28 {
        let x = 140.0 + i as f32 * 28.0 + ((i * 17) % 9) as f32;
        if x > 760.0 && x < 1060.0 {
            continue; // leave a gap around the tank so the host stays readable
        }
        let h = 46.0 + ((i * 13) % 22) as f32;
        let dark = rgba(28, 16, 40, 0.82);
        cx.ellipse(x, g - 8.0 - h, 11.0, h, dark);
        cx.circle(x, g - 8.0 - h * 2.0 + 8.0, 10.0 + ((i * 5) % 4) as f32, dark);
    }
    // ground
    cx.rect(-20.0, g, VW + 20.0, cx.vh + 40.0, 0.0, rgba(70, 110, 55, 1.0));
    cx.rect(-20.0, g, VW + 20.0, g + 14.0, 0.0, rgba(50, 80, 40, 0.7));
    // bunting + string lights across the top
    let yb = 36.0;
    cx.seg((0.0, yb + 8.0), (VW, yb + 18.0), 3.0, rgba(40, 20, 30, 0.7));
    for i in 0..22 {
        let u = i as f32 / 21.0;
        let x = 16.0 + u * (VW - 32.0);
        let y = yb + 8.0 + 10.0 * u + (if i % 2 == 0 { 10.0 } else { 0.0 });
        let cols = [
            rgba(220, 40, 60, 0.95),
            rgba(255, 200, 40, 0.95),
            rgba(50, 140, 220, 0.95),
            rgba(80, 200, 90, 0.95),
            rgba(240, 80, 180, 0.95),
        ];
        let col = cols[i % cols.len()];
        if i + 1 < 22 {
            let u2 = (i + 1) as f32 / 21.0;
            let x2 = 16.0 + u2 * (VW - 32.0);
            let y2 = yb + 8.0 + 10.0 * u2 + (if (i + 1) % 2 == 0 { 10.0 } else { 0.0 });
            cx.poly(&[(x, y), ((x + x2) * 0.5, y + 22.0), (x2, y2)], col);
        }
        let pulse = 0.55 + 0.45 * ((cx.time * 5.0 + i as f32 * 0.7).sin().abs());
        cx.glow(x, y, 16.0, rgba(255, 230, 140, 1.0), pulse);
        cx.circle(x, y, 5.0, rgba(255, 240, 180, pulse));
    }
}

fn draw_tank(cx: &Cx, g: f32, drop: f32, flash: f32) {
    let x = tank_x();
    let tank_top = g - 200.0;
    let tank_bot = g - 10.0;
    let tw = 150.0;
    // wooden booth frame around the tank
    cx.rect(x - tw - 22.0, tank_top - 18.0, x + tw + 22.0, tank_bot + 8.0, 6.0, rgba(90, 55, 35, 1.0));
    cx.rect(x - tw - 16.0, tank_top - 12.0, x + tw + 16.0, tank_bot + 2.0, 4.0, rgba(50, 90, 120, 1.0));
    // tiled glass wall
    cx.rect(x - tw, tank_top, x + tw, tank_bot, 6.0, rgba(90, 170, 200, 0.28));
    for i in 0..6 {
        let tx0 = x - tw + 4.0 + i as f32 * ((tw * 2.0 - 8.0) / 6.0);
        let tx1 = tx0 + (tw * 2.0 - 8.0) / 6.0;
        for j in 0..4 {
            let ty0 = tank_top + 6.0 + j as f32 * ((tank_bot - tank_top - 12.0) / 4.0);
            let ty1 = ty0 + (tank_bot - tank_top - 12.0) / 4.0;
            let tint = if (i + j) % 2 == 0 { rgba(70, 160, 200, 0.16) } else { rgba(40, 110, 160, 0.16) };
            cx.rect(tx0 + 1.0, ty0 + 1.0, tx1 - 1.0, ty1 - 1.0, 2.0, tint);
        }
    }
    cx.rect_line(x - tw, tank_top, x + tw, tank_bot, 6.0, 7.0, rgba(30, 70, 100, 1.0));
    // water + bubbles
    let fill = 0.58 + 0.12 * (cx.time * 1.5).sin();
    let wy = lerp(tank_bot, tank_top + 16.0, fill);
    cx.rect(x - tw + 6.0, wy, x + tw - 6.0, tank_bot - 4.0, 4.0, rgba(40, 140, 210, 0.72));
    cx.ellipse(x, wy, tw - 8.0, 12.0, rgba(120, 210, 240, 0.75));
    for i in 0..10 {
        let bx = x - tw * 0.7 + (i as f32 * 97.0 + cx.time * 18.0) % (tw * 1.4);
        let cycle = (cx.time * (0.35 + (i % 3) as f32 * 0.12) + i as f32 * 0.4).fract();
        let by = lerp(tank_bot - 12.0, wy + 8.0, cycle);
        let br = 3.5 + (i % 4) as f32 * 1.6;
        cx.circle(bx, by, br, rgba(200, 240, 255, 0.45 * (1.0 - cycle)));
        cx.circle(bx - br * 0.25, by - br * 0.25, br * 0.35, rgba(255, 255, 255, 0.5 * (1.0 - cycle)));
    }
    if drop > 0.4 {
        for i in 0..5 {
            let r = 24.0 + drop * 55.0 + i as f32 * 22.0;
            cx.ellipse(x, wy, r, 10.0 + i as f32 * 4.0, rgba(255, 255, 255, 0.28 * (1.0 - drop * 0.25)));
        }
    }
    // support posts + seat
    cx.rect(x - tw - 18.0, tank_top - 88.0, x - tw - 4.0, tank_bot, 3.0, rgba(90, 70, 50, 1.0));
    cx.rect(x + tw + 4.0, tank_top - 88.0, x + tw + 18.0, tank_bot, 3.0, rgba(90, 70, 50, 1.0));
    let seat_y = tank_top - 68.0 + drop * 40.0;
    let tilt = drop * 0.8;
    cx.rect(x - 54.0, seat_y, x + 54.0, seat_y + 16.0, 4.0, rgba(180, 100, 45, 1.0));
    if drop > 0.15 {
        let ang = tilt.min(1.0) * 1.2;
        let (sn, cs) = ang.sin_cos();
        let p0 = (x - 54.0, seat_y);
        let p1 = (x - 54.0 + 108.0 * cs, seat_y + 108.0 * sn);
        cx.seg(p0, p1, 14.0, rgba(180, 100, 45, 1.0));
    }
    // "ผู้ไลฟ์" sign on the seat post — host identity, stays readable while falling
    let sx0 = x + tw + 22.0;
    let sy0 = tank_top - 86.0;
    cx.rect(sx0 + 20.0, sy0 + 70.0, sx0 + 28.0, tank_top + 8.0, 2.0, rgba(80, 60, 40, 1.0));
    cx.rect(sx0, sy0, sx0 + 48.0 + 70.0, sy0 + 52.0, 8.0, rgba(30, 20, 40, 0.95));
    cx.rect_line(sx0, sy0, sx0 + 118.0, sy0 + 52.0, 8.0, 3.0, rgba(255, 80, 170, 0.95));
    cx.text(sx0 + 59.0, sy0 + 26.0, Align2::CENTER_CENTER, "ผู้ไลฟ์", 24.0, rgba(255, 230, 240, 1.0), true);
    // target pad
    let tx = x - tw - 36.0;
    let ty = tank_top - 10.0;
    if flash > 0.0 {
        let fa = clamp01(flash / 0.32);
        cx.glow(tx, ty, 70.0 + 40.0 * fa, rgba(255, 230, 80, 1.0), 0.85 * fa);
        cx.circle(tx, ty, 48.0 + 10.0 * fa, rgba(255, 240, 120, 0.55 * fa));
    }
    cx.circle(tx, ty, 40.0, rgba(240, 240, 240, 1.0));
    cx.circle(tx, ty, 28.0, rgba(220, 40, 40, 1.0));
    cx.circle(tx, ty, 16.0, rgba(240, 240, 240, 1.0));
    cx.circle(tx, ty, 6.0, rgba(220, 40, 40, 1.0));
    cx.rect(tx + 22.0, ty - 4.0, x - tw, ty + 4.0, 2.0, rgba(60, 60, 70, 1.0));
}

fn draw_meter(cx: &Cx, meter: f32, target: u32) {
    let x0 = 24.0;
    let y0 = 10.0;
    let w = 500.0;
    let h = 52.0;
    cx.rect(x0, y0, x0 + w, y0 + h, 12.0, rgba(10, 16, 36, 0.86));
    cx.rect_line(x0, y0, x0 + w, y0 + h, 12.0, 3.0, rgba(200, 230, 255, 0.85));
    let u = (meter / target.max(1) as f32).clamp(0.0, 1.0);
    let bar_x0 = x0 + 118.0;
    let bar_x1 = x0 + w - 96.0;
    cx.rect(bar_x0, y0 + 8.0, bar_x0 + (bar_x1 - bar_x0) * u.max(0.0), y0 + h - 8.0, 8.0, rgba(80, 200, 255, 0.98));
    if u > 0.02 {
        cx.rect(bar_x0, y0 + 8.0, bar_x0 + (bar_x1 - bar_x0) * u, y0 + 18.0, 8.0, rgba(200, 245, 255, 0.35));
    }
    cx.text(x0 + 58.0, y0 + h * 0.5, Align2::CENTER_CENTER, "เกจดังก์", 22.0, rgba(255, 240, 200, 1.0), true);
    let shown = meter.round() as u32;
    cx.text(x0 + w - 48.0, y0 + h * 0.5, Align2::CENTER_CENTER, &format!("{shown}/{target}"), 22.0, rgba(255, 255, 255, 1.0), true);
}

fn host_look() -> Look {
    // hot-pink / black streamer fit, tier 0 so no gold aura like mid-tier senders
    Look {
        tier: 0,
        shirt: Color32::from_rgb(255, 70, 160),
        pants: Color32::from_rgb(18, 16, 28),
        scale: 1.0,
        hair: Color32::from_rgb(28, 18, 20),
        style: 4,
    }
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
        assert!(st.meter() >= 10, "meter stays full during the dunk");
        assert!(matches!(s.phase(), Phase::Drop | Phase::Splash));
        // reset happens when the show finishes, not at the start of the drop
        let rest = s.total() - s.t + 0.05;
        st.tick(rest, 10, &ADD, &BALLS, 5, 6);
        assert_eq!(st.meter(), 0, "meter resets after the dunking show ends");
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
                assert!(st.meter() >= 10, "meter stays full while the host drops");
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
    fn meter_fills_per_hit_not_only_at_end() {
        let mut st = Stage::with_seed(55);
        st.enqueue(inn("จักรพรรดิเพชร", 4), 10, &ADD, &BALLS, 5, 6);
        // t5: 3 balls, all hit, first lands at walk + 0.18 + FLY
        st.tick(WALK + 0.18 + FLY - 0.08, 10, &ADD, &BALLS, 5, 6);
        assert_eq!(st.meter(), 0, "meter should still be empty mid-flight");
        st.tick(0.16, 10, &ADD, &BALLS, 5, 6);
        let after_first = st.meter();
        assert!(after_first > 0, "first hit must add to the meter immediately");
        // second ball t0 = 0.18 + THROW_EACH, lands later
        let t_now = st.lead().unwrap().t;
        let second_land = WALK + 0.18 + THROW_EACH + FLY + 0.05;
        if second_land > t_now {
            st.tick(second_land - t_now, 10, &ADD, &BALLS, 5, 6);
        }
        assert!(st.meter() > after_first, "second hit should increase the meter before the throw ends");
        assert!(!st.lead().unwrap().will_dunk || st.lead().unwrap().t >= st.lead().unwrap().walk + st.lead().unwrap().throw);
        let rem = st.lead().unwrap().walk + st.lead().unwrap().throw + 0.05 - st.lead().unwrap().t;
        if rem > 0.0 {
            st.tick(rem, 10, &ADD, &BALLS, 5, 6);
        }
        assert!(st.lead().unwrap().will_dunk);
        assert!(st.meter() >= 10);
    }

    #[test]
    fn same_sender_merges_throws() {
        let mut st = Stage::with_seed(2);
        st.set_flood(4.0, 20.0, 99);
        for _ in 0..6 {
            st.enqueue(inn("มะลิ", 0), 10, &ADD, &BALLS, 5, 6);
        }
        assert_eq!(st.len(), 1, "repeat throws merge into one show");
        assert_eq!(st.lead().unwrap().combo, 6);
        assert!(st.lead().unwrap().balls.len() >= 2, "merged gifts append balls");
    }

    #[test]
    fn dunk_party_and_overflow_are_bounded() {
        let mut st = Stage::with_seed(3);
        st.set_flood(4.0, 20.0, 6);
        for i in 0..20 {
            st.enqueue(inn(&format!("คน{i}"), 0), 10, &ADD, &BALLS, 5, 8);
        }
        assert!(st.in_party());
        assert!(st.shows().count() <= 8);
        assert!(st.live_units() <= 200, "got {}", st.live_units());
        st.tick(21.0, 10, &ADD, &BALLS, 5, 8);
        assert!(!st.in_party());
        assert_eq!(st.waiting_extra(), 0);
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
