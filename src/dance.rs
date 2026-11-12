//! โหมดเต้น — ของขวัญหนึ่งชิ้น = ภารกิจเต้นหนึ่งใบ (กิมมิก + นับถอยหลัง + ตัวชี้ท่า)
//! ฉากเวทีหลายแบบเลือกตามขั้น / สุ่ม / ล็อกในตั้งค่า  ฟลอร์รวมให้ทั้งคิวขึ้นเต้นด้วยกัน
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::VecDeque;
use std::f32::consts::{PI, TAU};

pub const WALK: f32 = 1.65;
pub const LEAVE: f32 = 1.55;
const DEFAULT_COUNTDOWN: [f32; 5] = [8.0, 10.0, 14.0, 18.0, 24.0];

pub fn floor_y(vh: f32) -> f32 {
    vh * 0.86
}

/// Built-in dance venues. Higher `min_tier` = grander, unlocked later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Venue {
    /// ฟลอร์รวม — every queued sender dances together
    Floor,
    /// ลานวัด / งานวัด
    TempleFair,
    /// เต้นบนโต๊ะ
    Table,
    /// บนลำโพง
    Speaker,
    /// บนเวที คอนเสิร์ต ไฟ + ควัน
    Stage,
    /// รถแห่ / รถสิบล้อเครื่องเสียง
    Parade,
    /// บนดาดฟ้ายามค่ำ
    Rooftop,
    /// เต้นใต้ฝนโฟม
    Foam,
    /// บนเมฆ
    Cloud,
}

impl Venue {
    pub const ALL: [Venue; 9] = [
        Venue::Floor,
        Venue::TempleFair,
        Venue::Table,
        Venue::Speaker,
        Venue::Stage,
        Venue::Parade,
        Venue::Rooftop,
        Venue::Foam,
        Venue::Cloud,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Venue::Floor => "floor",
            Venue::TempleFair => "temple_fair",
            Venue::Table => "table",
            Venue::Speaker => "speaker",
            Venue::Stage => "stage",
            Venue::Parade => "parade",
            Venue::Rooftop => "rooftop",
            Venue::Foam => "foam",
            Venue::Cloud => "cloud",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Venue::Floor => "ฟลอร์รวม",
            Venue::TempleFair => "ลานวัด / งานวัด",
            Venue::Table => "เต้นบนโต๊ะ",
            Venue::Speaker => "บนลำโพง",
            Venue::Stage => "บนเวที",
            Venue::Parade => "รถแห่เครื่องเสียง",
            Venue::Rooftop => "บนดาดฟ้ายามค่ำ",
            Venue::Foam => "เต้นใต้ฝนโฟม",
            Venue::Cloud => "บนเมฆ",
        }
    }

    /// 0-based gift tier that unlocks this venue when selection is auto/random.
    pub fn min_tier(self) -> usize {
        match self {
            Venue::Floor | Venue::TempleFair => 0,
            Venue::Table | Venue::Speaker => 1,
            Venue::Stage | Venue::Parade => 2,
            Venue::Rooftop | Venue::Foam => 3,
            Venue::Cloud => 4,
        }
    }

    pub fn from_key(s: &str) -> Option<Venue> {
        let k = s.trim().to_lowercase();
        // Thai aliases too
        match k.as_str() {
            "floor" | "ฟลอร์รวม" | "ฟลอร์" => Some(Venue::Floor),
            "temple_fair" | "temple" | "fair" | "ลานวัด" | "งานวัด" => Some(Venue::TempleFair),
            "table" | "เต้นบนโต๊ะ" | "โต๊ะ" => Some(Venue::Table),
            "speaker" | "บนลำโพง" | "ลำโพง" => Some(Venue::Speaker),
            "stage" | "บนเวที" | "เวที" => Some(Venue::Stage),
            "parade" | "รถแห่" | "รถสิบล้อ" | "รถสิบล้อเครื่องเสียง" | "รถแห่เครื่องเสียง" => Some(Venue::Parade),
            "rooftop" | "บนดาดฟ้า" | "ดาดฟ้า" | "บนดาดฟ้ายามค่ำ" => Some(Venue::Rooftop),
            "foam" | "ฝนโฟม" | "เต้นใต้ฝนโฟม" => Some(Venue::Foam),
            "cloud" | "บนเมฆ" | "เมฆ" => Some(Venue::Cloud),
            _ => None,
        }
    }
}

pub fn default_challenges() -> Vec<String> {
    [
        "เต้นหุ่นยนต์",
        "เต้นสโลว์โมชัน",
        "สะท้อนการ์ตูนเป็นกระจก",
        "โพสท่าแล้วค้าง",
        "เต้นไก่",
        "เต้นกับพัด",
        "หมุนตัวแล้วชี้ฟ้า",
        "กระโดดตามจังหวะ",
        "โบกแขนเป็นคลื่น",
        "ก้าวปูข้าง ๆ",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveKind {
    Robot,
    SlowMo,
    Mirror,
    Freeze,
    Chicken,
    Fan,
    Spin,
    Jump,
    Wave,
    Crab,
}

impl MoveKind {
    pub fn from_text(s: &str) -> Option<MoveKind> {
        if s.contains("หุ่นยนต์") || s.contains("โรบอท") {
            Some(MoveKind::Robot)
        } else if s.contains("สโลว์") || s.contains("ช้า") {
            Some(MoveKind::SlowMo)
        } else if s.contains("กระจก") || s.contains("สะท้อน") {
            Some(MoveKind::Mirror)
        } else if s.contains("ค้าง") || s.contains("โพส") {
            Some(MoveKind::Freeze)
        } else if s.contains("ไก่") {
            Some(MoveKind::Chicken)
        } else if s.contains("พัด") || s.contains("พร็อพ") || s.contains("หมวก") || s.contains("ไมค์") {
            Some(MoveKind::Fan)
        } else if s.contains("หมุน") || s.contains("ชี้ฟ้า") {
            Some(MoveKind::Spin)
        } else if s.contains("กระโดด") || s.contains("บีท") || s.contains("จังหวะ") {
            Some(MoveKind::Jump)
        } else if s.contains("คลื่น") || s.contains("โบก") {
            Some(MoveKind::Wave)
        } else if s.contains("ปู") {
            Some(MoveKind::Crab)
        } else {
            None
        }
    }
}

pub fn parse_moves(title: &str, joiner: &str) -> Vec<MoveKind> {
    let parts: Vec<&str> = if joiner.is_empty() {
        vec![title]
    } else {
        title.split(joiner).collect()
    };
    let mut out = Vec::new();
    for p in parts {
        if let Some(m) = MoveKind::from_text(p) {
            if !out.contains(&m) {
                out.push(m);
            }
        }
    }
    if out.is_empty() {
        out.push(MoveKind::Wave);
    }
    out
}

pub fn combo_challenge(a: &str, b: &str, joiner: &str) -> String {
    format!("{a}{joiner}{b}")
}

pub fn countdown_for(list: &[f32], tier: usize) -> f32 {
    let t = tier.min(4);
    list.get(t).copied().filter(|s| *s > 0.5).unwrap_or(DEFAULT_COUNTDOWN[t])
}

/// Pick a venue: a concrete key forces that venue; "auto"/"random"/empty pick among
/// enabled venues unlocked at `tier` (auto weights toward grander stages).
pub fn pick_venue(mode: &str, enabled: &[String], tier: usize, rng: &mut fastrand::Rng) -> Venue {
    if let Some(v) = Venue::from_key(mode) {
        return v;
    }
    let catalog = enabled_venues(enabled);
    let unlocked: Vec<Venue> = catalog.into_iter().filter(|v| v.min_tier() <= tier).collect();
    if unlocked.is_empty() {
        return Venue::Floor;
    }
    let auto = mode.trim().is_empty() || mode.eq_ignore_ascii_case("auto") || mode == "อัตโนมัติ" || mode == "ตามขั้น";
    let equal = mode.eq_ignore_ascii_case("random") || mode == "สุ่ม";
    if equal {
        return unlocked[rng.usize(0..unlocked.len())];
    }
    if auto {
        // weight 1 + 2*min_tier so t5 prefers cloud/foam/rooftop
        let mut pick = unlocked[0];
        let mut sum = 0.0f32;
        for v in &unlocked {
            sum += 1.0 + 2.0 * v.min_tier() as f32;
        }
        let mut r = rng.f32() * sum;
        for v in unlocked {
            r -= 1.0 + 2.0 * v.min_tier() as f32;
            if r <= 0.0 {
                pick = v;
                break;
            }
        }
        return pick;
    }
    Venue::Floor
}

pub fn enabled_venues(enabled: &[String]) -> Vec<Venue> {
    if enabled.is_empty() {
        return Venue::ALL.to_vec();
    }
    let mut out: Vec<Venue> = enabled.iter().filter_map(|k| Venue::from_key(k)).collect();
    out.dedup();
    if out.is_empty() { Venue::ALL.to_vec() } else { out }
}

pub struct DanceIn {
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub gift: String,
}

#[derive(Clone, Debug)]
pub struct Show {
    pub user: String,
    pub tier: usize,
    pub tier_name: String,
    pub gift: String,
    pub challenge: String,
    pub moves: Vec<MoveKind>,
    pub venue: Venue,
    pub t: f32,
    pub walk: f32,
    pub dance: f32,
    pub leave: f32,
    pub look: Look,
    pub seed: u64,
    pub x: f32,
    pub appear: f32,
    pub combo: u32,
    last_gift_at: f32,
}

impl Show {
    pub fn is_done(&self) -> bool {
        self.t >= self.walk + self.dance + self.leave
    }
    pub fn phase_name(&self) -> &'static str {
        if self.t < self.walk {
            "walk"
        } else if self.t < self.walk + self.dance {
            "dance"
        } else if self.t < self.walk + self.dance + self.leave {
            "leave"
        } else {
            "done"
        }
    }
    pub fn dance_t(&self) -> f32 {
        (self.t - self.walk).max(0.0)
    }
    pub fn remain(&self) -> f32 {
        (self.walk + self.dance - self.t).max(0.0)
    }
    pub fn in_dance(&self) -> bool {
        self.t >= self.walk && self.t < self.walk + self.dance
    }
}

struct Bit {
    kind: u8, // 0 confetti, 1 foam, 2 spark, 3 note
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    hue: f32,
    s: f32,
}

struct Pending {
    inn: DanceIn,
    challenge: String,
    moves: Vec<MoveKind>,
    venue: Venue,
    combo: u32,
    last_gift_at: f32,
}

struct Party {
    remain: f32,
    challenge: String,
    names: VecDeque<String>,
    gifts: u32,
    best_tier: usize,
}

const MAX_RECENT: usize = 64;
const MAX_TICKER: usize = 48;
const MAX_WAITING: usize = 24;
const MAX_OVERFLOW_NAMES: usize = 64;
const MAX_BITS: usize = 180;

pub struct Stage {
    rng: fastrand::Rng,
    shows: VecDeque<Show>,
    waiting: VecDeque<Pending>,
    overflow: VecDeque<String>,
    overflow_extra: u32,
    recent: VecDeque<(f32, String)>,
    party: Option<Party>,
    merge_window: f32,
    max_seconds: f32,
    party_threshold: u32,
    pub started: u64,
    pub finished: u64,
    bits: Vec<Bit>,
    clock: f32,
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
            overflow: VecDeque::new(),
            overflow_extra: 0,
            recent: VecDeque::new(),
            party: None,
            merge_window: 4.0,
            max_seconds: 20.0,
            party_threshold: 8,
            started: 0,
            finished: 0,
            bits: vec![],
            clock: 0.0,
        }
    }
    pub fn with_seed(seed: u64) -> Self {
        let mut s = Self::new();
        s.rng = fastrand::Rng::with_seed(seed);
        s
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
    /// Bound on live records (for flood tests). Caps keep this from growing with gift spam.
    pub fn live_units(&self) -> usize {
        self.shows.len()
            + self.waiting.len()
            + self.overflow.len()
            + self.recent.len()
            + self.bits.len()
            + self.party.as_ref().map(|p| p.names.len()).unwrap_or(0)
    }
    pub fn clear(&mut self) {
        self.shows.clear();
        self.waiting.clear();
        self.overflow.clear();
        self.overflow_extra = 0;
        self.recent.clear();
        self.party = None;
        self.bits.clear();
        self.started = 0;
        self.finished = 0;
    }
    pub fn busy(&self) -> bool {
        !self.shows.is_empty() || !self.waiting.is_empty() || !self.overflow.is_empty() || self.overflow_extra > 0 || self.party.is_some()
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
    /// Names for the "ถัดไป" HUD. On ฟลอร์รวม everyone already on stage is dancing,
    /// so only list senders still waiting off-stage.
    pub fn hud_waiting_names(&self) -> Vec<&str> {
        if self.active_venue() == Some(Venue::Floor) || self.party.is_some() {
            let mut v: Vec<&str> = self.waiting.iter().map(|w| w.inn.user.as_str()).collect();
            v.extend(self.overflow.iter().map(|s| s.as_str()));
            v
        } else {
            self.waiting_names()
        }
    }
    pub fn active_venue(&self) -> Option<Venue> {
        self.shows.front().map(|s| s.venue)
    }

    pub fn enqueue(
        &mut self,
        inn: DanceIn,
        challenges: &[String],
        combo_joiner: &str,
        venue_mode: &str,
        venues: &[String],
        countdown: &[f32],
        max_visible: usize,
    ) {
        self.note_gift(&inn.user);
        let cap = max_visible.max(1).min(12);
        let extend = (countdown_for(countdown, inn.tier) * 0.35).clamp(2.0, 6.0);

        // same sender within the merge window → one card with ×N, longer (capped) time
        if self.try_merge(&inn, extend) {
            if self.recent_count() >= self.party_threshold as usize && self.party.is_none() {
                self.enter_party(&inn, challenges, combo_joiner, countdown, cap);
            }
            return;
        }

        if self.party.is_some() || self.recent_count() >= self.party_threshold as usize {
            self.enter_party(&inn, challenges, combo_joiner, countdown, cap);
            self.place_party_dancer(inn, challenges, combo_joiner, countdown, cap);
            return;
        }

        let (challenge, moves) = pick_challenge(&mut self.rng, inn.tier, challenges, combo_joiner);
        let venue = pick_venue(venue_mode, venues, inn.tier, &mut self.rng);
        if self.shows.len() < cap {
            let show = self.make_show(inn, challenge, moves, venue, countdown);
            self.started += 1;
            self.shows.push_back(show);
        } else {
            self.push_waiting(Pending { inn, challenge, moves, venue, combo: 1, last_gift_at: self.clock });
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

    fn try_merge(&mut self, inn: &DanceIn, extend: f32) -> bool {
        let window = self.merge_window;
        let cap_s = self.max_seconds;
        let clock = self.clock;
        if let Some(s) = self.shows.iter_mut().find(|s| s.user == inn.user && clock - s.last_gift_at <= window) {
            s.combo = s.combo.saturating_add(1);
            s.last_gift_at = clock;
            s.dance = (s.dance + extend).min(cap_s);
            if inn.tier > s.tier {
                s.tier = inn.tier;
                s.tier_name = inn.tier_name.clone();
                s.look = appearance_for(&s.user, s.tier);
            }
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

    fn ticker_push(names: &mut VecDeque<String>, user: &str) {
        if names.back().map(|s| s == user).unwrap_or(false) {
            return;
        }
        names.push_back(user.to_string());
        while names.len() > MAX_TICKER {
            names.pop_front();
        }
    }

    fn enter_party(&mut self, inn: &DanceIn, challenges: &[String], combo_joiner: &str, countdown: &[f32], cap: usize) {
        let extend = (countdown_for(countdown, inn.tier) * 0.35).clamp(2.0, 6.0);
        if let Some(p) = &mut self.party {
            p.gifts = p.gifts.saturating_add(1);
            p.remain = (p.remain + extend).min(self.max_seconds);
            p.best_tier = p.best_tier.max(inn.tier);
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
        let gifts = self.recent_count().max(1) as u32;
        for s in &mut self.shows {
            s.venue = Venue::Floor;
        }
        let challenge = if let Some(s) = self.shows.front() {
            s.challenge.clone()
        } else {
            let (c, _) = pick_challenge(&mut self.rng, inn.tier, challenges, combo_joiner);
            c
        };
        let remain = (12.0 + gifts as f32 * 0.35).clamp(8.0, self.max_seconds);
        self.party = Some(Party { remain, challenge, names, gifts, best_tier: inn.tier });
        let _ = cap;
    }

    fn place_party_dancer(&mut self, inn: DanceIn, challenges: &[String], combo_joiner: &str, countdown: &[f32], cap: usize) {
        if self.shows.iter().any(|s| s.user == inn.user) {
            return;
        }
        if self.waiting.iter().any(|w| w.inn.user == inn.user) {
            return;
        }
        let (challenge, moves) = pick_challenge(&mut self.rng, inn.tier, challenges, combo_joiner);
        if self.shows.len() < cap {
            let mut show = self.make_show(inn, challenge, moves, Venue::Floor, countdown);
            if show.t < show.walk {
                // join mid-party: walk in, then dance
            }
            self.started += 1;
            self.shows.push_back(show);
        } else {
            self.push_waiting(Pending { inn, challenge, moves, venue: Venue::Floor, combo: 1, last_gift_at: self.clock });
        }
    }

    fn push_waiting(&mut self, pend: Pending) {
        if self.waiting.len() < MAX_WAITING {
            self.waiting.push_back(pend);
            return;
        }
        // compact: keep the name, drop the bulky pending record
        if self.overflow.len() < MAX_OVERFLOW_NAMES {
            self.overflow.push_back(pend.inn.user);
        } else {
            self.overflow_extra = self.overflow_extra.saturating_add(1);
        }
    }

    fn pop_waiting(&mut self, countdown: &[f32]) -> Option<(DanceIn, String, Vec<MoveKind>, Venue, u32)> {
        if let Some(w) = self.waiting.pop_front() {
            return Some((w.inn, w.challenge, w.moves, w.venue, w.combo));
        }
        if let Some(user) = self.overflow.pop_front() {
            let inn = DanceIn { user, tier: 0, tier_name: "ขั้น1".into(), gift: String::new() };
            let dance_t = countdown_for(countdown, 0);
            let _ = dance_t;
            return Some((inn, "เต้นตามจังหวะ".into(), vec![MoveKind::Wave], Venue::Floor, 1));
        }
        None
    }

    fn end_party(&mut self) {
        let extra = self.waiting_extra() as u64;
        self.waiting.clear();
        self.overflow.clear();
        self.overflow_extra = 0;
        self.finished += extra;
        for s in &mut self.shows {
            if s.t < s.walk + s.dance {
                s.t = s.walk + s.dance;
            }
        }
        self.party = None;
    }

    fn make_show(&mut self, inn: DanceIn, challenge: String, moves: Vec<MoveKind>, venue: Venue, countdown: &[f32]) -> Show {
        let seed = self.rng.u64(..);
        let dance = countdown_for(countdown, inn.tier).min(self.max_seconds);
        let look = appearance_for(&inn.user, inn.tier);
        Show {
            user: inn.user,
            tier: inn.tier,
            tier_name: inn.tier_name,
            gift: inn.gift,
            challenge,
            moves,
            venue,
            t: 0.0,
            walk: WALK,
            dance,
            leave: LEAVE,
            look,
            seed,
            x: -80.0,
            appear: 0.0,
            combo: 1,
            last_gift_at: self.clock,
        }
    }

    pub fn tick(&mut self, dt: f32, max_visible: usize, countdown: &[f32]) {
        self.clock += dt;
        let cap = max_visible.max(1).min(12);
        let in_party = self.party.is_some();
        let floor_all = in_party || self.active_venue() == Some(Venue::Floor);

        if let Some(p) = &mut self.party {
            p.remain -= dt;
        }

        if in_party {
            for s in &mut self.shows {
                if s.t < s.walk {
                    s.t = (s.t + dt).min(s.walk);
                }
                s.appear = (s.appear + dt / 0.35).min(1.0);
            }
            if self.party.as_ref().map(|p| p.remain <= 0.0).unwrap_or(false) {
                self.end_party();
            }
        } else {
            // lead advances fully; backups walk in then hold (on ฟลอร์รวม they keep dancing visually via clock)
            if let Some(lead) = self.shows.front_mut() {
                lead.t += dt;
                lead.appear = (lead.appear + dt / 0.35).min(1.0);
            }
            for (i, s) in self.shows.iter_mut().enumerate() {
                if i == 0 {
                    continue;
                }
                if s.t < s.walk {
                    s.t = (s.t + dt).min(s.walk);
                }
                s.appear = (s.appear + dt / 0.35).min(1.0);
            }
            if self.shows.front().map(|s| s.is_done()).unwrap_or(false) {
                self.finished += 1;
                self.shows.pop_front();
                if let Some(next) = self.shows.front_mut() {
                    if next.t >= next.walk {
                        next.t = next.walk;
                    }
                }
            }
        }

        while self.shows.len() < cap {
            let Some((inn, challenge, moves, venue, combo)) = self.pop_waiting(countdown) else { break };
            let mut show = self.make_show(inn, challenge, moves, if in_party || floor_all { Venue::Floor } else { venue }, countdown);
            show.combo = combo.max(1);
            self.started += 1;
            self.shows.push_back(show);
        }
        // ambient bits
        let lead_tier = self.shows.front().map(|s| s.tier).unwrap_or(0);
        let venue = self.active_venue();
        if let Some(v) = venue {
            self.spawn_bits(dt, v, lead_tier);
        }
        for b in &mut self.bits {
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.vy += if b.kind == 1 { 40.0 } else { 180.0 } * dt;
            if b.kind == 3 {
                b.vy = -30.0;
                b.x += (self.clock * 3.0 + b.hue).sin() * 20.0 * dt;
            }
            b.life -= dt;
        }
        self.bits.retain(|b| b.life > 0.0 && b.y < 2000.0);
        while self.bits.len() > MAX_BITS {
            self.bits.remove(0);
        }
    }

    fn spawn_bits(&mut self, dt: f32, venue: Venue, tier: usize) {
        let dancing = self.party.is_some() || self.shows.front().map(|s| s.in_dance()).unwrap_or(false);
        if !dancing {
            return;
        }
        let confetti_rate = [0.0, 0.5, 2.0, 6.0, 14.0][tier.min(4)];
        if confetti_rate > 0.0 && self.rng.f32() < confetti_rate * dt {
            for _ in 0..(1 + tier / 2) {
                self.bits.push(Bit {
                    kind: 0,
                    x: 80.0 + self.rng.f32() * 1120.0,
                    y: -20.0 + self.rng.f32() * 80.0,
                    vx: self.rng.f32() * 160.0 - 80.0,
                    vy: 40.0 + self.rng.f32() * 80.0,
                    life: 2.2,
                    max: 2.2,
                    hue: self.rng.f32() * 360.0,
                    s: 5.0 + self.rng.f32() * 7.0,
                });
            }
        }
        if venue == Venue::Foam && self.rng.f32() < 18.0 * dt {
            self.bits.push(Bit {
                kind: 1,
                x: 40.0 + self.rng.f32() * 1200.0,
                y: -10.0,
                vx: self.rng.f32() * 40.0 - 20.0,
                vy: 80.0 + self.rng.f32() * 60.0,
                life: 3.0,
                max: 3.0,
                hue: 0.0,
                s: 8.0 + self.rng.f32() * 22.0,
            });
        }
        if tier >= 2 && self.rng.f32() < 3.0 * dt {
            self.bits.push(Bit {
                kind: 3,
                x: 200.0 + self.rng.f32() * 800.0,
                y: floor_y(720.0) - 280.0 - self.rng.f32() * 80.0,
                vx: 0.0,
                vy: -20.0,
                life: 1.6,
                max: 1.6,
                hue: self.rng.f32() * TAU,
                s: 10.0,
            });
        }
    }

    pub fn draw_world(&self, cx: &Cx, pcx: &Cx, sprites: &Sprites, bg: &str, max_visible: usize) {
        let g = floor_y(cx.vh);
        let venue = if self.party.is_some() { Venue::Floor } else { self.active_venue().unwrap_or(Venue::Floor) };
        let lead_tier = self.party.as_ref().map(|p| p.best_tier).unwrap_or_else(|| self.shows.front().map(|s| s.tier).unwrap_or(0));
        let dancing = self.party.is_some() || self.shows.front().map(|s| s.in_dance() || s.phase_name() == "leave").unwrap_or(false);
        if bg == "scene" {
            draw_venue_sky(cx, venue, lead_tier);
        }
        draw_venue_set(if bg == "scene" { cx } else { pcx }, venue, g, lead_tier, dancing, bg);
        if self.party.is_some() {
            draw_party_disco(cx, g, self.clock);
        }
        // characters back-to-front (lead last)
        let n = self.shows.len().max(1);
        // draw backups first
        for i in 1..self.shows.len() {
            self.draw_one(cx, sprites, i, n, venue, g, false);
        }
        if let Some(_) = self.shows.front() {
            self.draw_one(cx, sprites, 0, n, venue, g, true);
        }
        self.draw_bits(cx, venue);
        // t5 crowd in front of the floor edge
        if lead_tier >= 4 && bg == "scene" {
            draw_crowd(cx, g, cx.time);
        }
        let _ = max_visible;
    }

    fn draw_one(&self, cx: &Cx, sprites: &Sprites, i: usize, n: usize, venue: Venue, g: f32, is_lead: bool) {
        let s = &self.shows[i];
        let (tx, gy) = slot(venue, i, n, is_lead, g, cx.time, s);
        let (x, pose, look) = self.pose_at(s, tx, is_lead, venue);
        let mut dcy = cx.with_painter(cx.p);
        dcy.gy = Some(gy);
        let rig = draw_character(&dcy, sprites, x, &pose, &look);
        if current_move(&s.moves, s.dance_t(), s.dance) == MoveKind::Fan && s.t >= s.walk {
            draw_fan(&dcy, rig.arms[1].1, cx.time + s.seed as f32 * 0.01);
        }
        // name tag well above the head, never over the face
        let tag_y = rig.head.1 - HEAD_R * look.scale - 18.0;
        let alpha = if s.phase_name() == "leave" {
            1.0 - clamp01((s.t - s.walk - s.dance) / s.leave)
        } else {
            s.appear
        };
        draw_name_banner(cx, x, tag_y, &s.user, s.tier, if is_lead { &s.tier_name } else { "" }, s.appear, alpha);
    }

    fn pose_at(&self, s: &Show, target_x: f32, is_lead: bool, venue: Venue) -> (f32, Pose, Look) {
        let mut look = Look { scale: if is_lead { 1.0 + 0.04 * s.tier as f32 } else { 0.86 }, ..s.look };
        if venue == Venue::Speaker && is_lead {
            look.scale *= 0.92;
        }
        let leave_u = if s.t >= s.walk + s.dance {
            clamp01((s.t - s.walk - s.dance) / s.leave)
        } else {
            0.0
        };
        let walk_u = clamp01(s.t / s.walk);
        let x = if s.t < s.walk {
            lerp(-100.0, target_x, ease_in_out(walk_u))
        } else if leave_u > 0.0 {
            lerp(target_x, VW + 120.0, ease_in_out(leave_u))
        } else {
            target_x
        };
        let pose = if s.t < s.walk {
            let mut p = Pose::default();
            p.walk = 1.0;
            p.phase = walk_u * 8.0;
            p.facing = 1.0;
            p.breath = (s.t * 2.0).sin() * 0.4;
            p
        } else if leave_u > 0.0 {
            let mut p = Pose::default();
            p.walk = 1.0;
            p.phase = leave_u * 7.0;
            p.facing = 1.0;
            p.expr = 1;
            p
        } else {
            let dance_t = if is_lead && self.party.is_none() {
                s.dance_t()
            } else {
                // ฟลอร์รวม backups (and everyone in party) keep moving, not frozen on frame 0
                self.clock + (s.seed as f32) * 0.01
            };
            let mv = current_move(&s.moves, dance_t, s.dance.max(0.5));
            let mut p = pose_for(mv, dance_t, (s.seed as f32) * 0.001 + i_hash(&s.user) as f32 * 0.01, is_lead);
            // crab sidestep around the slot
            if mv == MoveKind::Crab {
                // x is adjusted below
            }
            p.breath = (s.t * 3.0).sin() * 0.5;
            p
        };
        let mut x = x;
        if s.in_dance() && current_move(&s.moves, s.dance_t(), s.dance) == MoveKind::Crab {
            x += (s.dance_t() * 3.2 + (s.seed % 50) as f32 * 0.1).sin() * 36.0;
        }
        if s.in_dance() && current_move(&s.moves, s.dance_t(), s.dance) == MoveKind::Jump {
            // hop is in squash; also a little x bounce is enough
        }
        let _ = pose.facing;
        (x, pose, look)
    }

    fn draw_bits(&self, cx: &Cx, venue: Venue) {
        for b in &self.bits {
            let a = clamp01(b.life / b.max.max(0.01));
            match b.kind {
                1 => {
                    cx.ellipse(b.x, b.y, b.s, b.s * 0.7, rgba(255, 255, 255, 0.55 * a));
                    cx.ellipse(b.x - b.s * 0.3, b.y - b.s * 0.2, b.s * 0.5, b.s * 0.4, rgba(255, 255, 255, 0.35 * a));
                }
                3 => {
                    // eighth note
                    let c = rgba(255, 230, 120, 0.85 * a);
                    cx.circle(b.x, b.y, 6.0, c);
                    cx.seg((b.x + 5.0, b.y), (b.x + 5.0, b.y - 22.0), 2.5, c);
                }
                _ => {
                    let c = with_a(hsv(b.hue, 0.75, 1.0), 0.9 * a);
                    cx.rect(b.x, b.y, b.x + b.s, b.y + b.s * 0.6, 1.0, c);
                }
            }
        }
        let _ = venue;
    }

    pub fn draw_overlay(&self, cx: &Cx, card_title: &str, guide_label: &str, sprites: &Sprites) {
        if let Some(p) = &self.party {
            self.draw_party_overlay(cx, p);
            self.draw_waiting_badge(cx);
            return;
        }
        self.draw_waiting_badge(cx);
        let Some(s) = self.shows.front() else { return };
        if !s.in_dance() && s.phase_name() != "leave" {
            return;
        }
        let fade = if s.in_dance() {
            clamp01(s.dance_t() / 0.35) * (1.0 - clamp01((s.remain()) / 0.01).min(1.0) * 0.0)
                * if s.remain() < 0.4 { s.remain() / 0.4 } else { 1.0 }
        } else {
            1.0 - clamp01((s.t - s.walk - s.dance) / 0.5)
        };
        if fade < 0.02 {
            return;
        }
        // task card — wide, large type so the challenge stays readable on a phone overlay
        let remain = s.remain().ceil() as i32;
        let card_w = 1120.0;
        let card_x = VW * 0.5;
        let card_y = 10.0;
        let h = 250.0;
        cx.rect(card_x - card_w * 0.5, card_y, card_x + card_w * 0.5, card_y + h, 22.0, rgba(12, 8, 28, 0.88 * fade));
        cx.rect_line(card_x - card_w * 0.5, card_y, card_x + card_w * 0.5, card_y + h, 22.0, 4.0, rgba(255, 210, 80, 0.85 * fade));
        cx.text(card_x, card_y + 24.0, Align2::CENTER_CENTER, card_title, 26.0, rgba(255, 230, 140, fade), true);
        // sender name is the second-most important line
        cx.text(card_x, card_y + 62.0, Align2::CENTER_CENTER, &s.user, 40.0, rgba(255, 255, 255, fade), true);
        if s.combo >= 2 {
            let badge = format!("×{}", s.combo);
            let nm = cx.measure(&s.user, 40.0, true);
            let bx = card_x + nm.x * 0.5 + 48.0;
            let by = card_y + 62.0;
            cx.rect(bx - 38.0, by - 28.0, bx + 38.0, by + 28.0, 14.0, rgba(220, 40, 80, 0.95 * fade));
            cx.rect_line(bx - 38.0, by - 28.0, bx + 38.0, by + 28.0, 14.0, 3.0, rgba(255, 230, 80, 0.95 * fade));
            cx.text(bx, by, Align2::CENTER_CENTER, &badge, 36.0, rgba(255, 255, 255, fade), true);
        }
        cx.text(card_x, card_y + 96.0, Align2::CENTER_CENTER, s.venue.label(), 22.0, rgba(255, 220, 160, 0.9 * fade), true);
        // challenge is the key info — large + bold, wraps inside the wide card
        cx.text_wrapped(card_x, card_y + 116.0, &s.challenge, 48.0, card_w - 64.0, rgba(255, 245, 210, fade), true);
        let count = format!("{remain}");
        let pulse = 1.0 + 0.07 * (cx.time * 8.0).sin();
        cx.text(card_x, card_y + h - 32.0, Align2::CENTER_CENTER, &count, 64.0 * pulse, rgba(255, 80, 120, fade), true);

        // guide character — right side, looping the move, labelled ตัวชี้ท่า
        let venue = s.venue;
        let g = floor_y(cx.vh);
        let (gx, gy) = guide_slot(venue, g);
        let mv = current_move(&s.moves, s.dance_t(), s.dance);
        let pose = pose_for(mv, cx.time * if mv == MoveKind::SlowMo { 0.5 } else { 1.0 }, 0.4, true);
        let look = Look {
            tier: 2,
            shirt: hsv(320.0, 0.45, 0.95),
            pants: hsv(210.0, 0.35, 0.35),
            scale: 0.78,
            hair: hsv(30.0, 0.55, 0.35),
            style: 2,
        };
        let mut gcx = cx.with_painter(cx.p);
        gcx.gy = Some(gy);
        let rig = draw_character(&gcx, sprites, gx, &pose, &look);
        if mv == MoveKind::Fan {
            draw_fan(&gcx, rig.arms[1].1, cx.time);
        }
        let ly = rig.head.1 - HEAD_R * look.scale - 10.0;
        // small label, not a full name banner (avoids fighting the sender tag)
        let lab = guide_label;
        let m = cx.measure(lab, 18.0, true);
        let lw = m.x + 18.0;
        cx.rect(gx - lw * 0.5, ly - 16.0, gx + lw * 0.5, ly + 8.0, 8.0, rgba(40, 20, 70, 0.8 * fade));
        cx.text(gx, ly - 4.0, Align2::CENTER_CENTER, lab, 18.0, rgba(255, 220, 255, fade), true);
    }

    fn draw_waiting_badge(&self, cx: &Cx) {
        let n = self.waiting_extra();
        if n == 0 {
            return;
        }
        let txt = format!("+{n} คนกำลังรอ");
        let size = 32.0;
        let m = cx.measure(&txt, size, true);
        let x1 = VW - 24.0;
        let y1 = cx.vh - 80.0;
        let x0 = x1 - m.x - 40.0;
        let y0 = y1 - 52.0;
        cx.rect(x0, y0, x1, y1, 16.0, rgba(20, 8, 40, 0.82));
        cx.rect_line(x0, y0, x1, y1, 16.0, 3.0, rgba(255, 200, 80, 0.9));
        cx.text((x0 + x1) * 0.5, (y0 + y1) * 0.5, Align2::CENTER_CENTER, &txt, size, rgba(255, 255, 255, 1.0), true);
    }

    fn draw_party_overlay(&self, cx: &Cx, p: &Party) {
        let card_w = 1120.0;
        let card_x = VW * 0.5;
        let card_y = 10.0;
        let h = 230.0;
        cx.rect(card_x - card_w * 0.5, card_y, card_x + card_w * 0.5, card_y + h, 22.0, rgba(20, 4, 40, 0.9));
        cx.rect_line(card_x - card_w * 0.5, card_y, card_x + card_w * 0.5, card_y + h, 22.0, 5.0, rgba(255, 80, 200, 0.95));
        let flash = 0.75 + 0.25 * (cx.time * 8.0).sin().abs();
        cx.text(card_x, card_y + 28.0, Align2::CENTER_CENTER, "โหมดปาร์ตี้!", 40.0, rgba(255, 80, 200, flash), true);
        cx.text_wrapped(card_x, card_y + 80.0, &p.challenge, 44.0, card_w - 64.0, rgba(255, 245, 210, 1.0), true);
        let remain = p.remain.ceil() as i32;
        let pulse = 1.0 + 0.07 * (cx.time * 8.0).sin();
        cx.text(card_x, card_y + h - 36.0, Align2::CENTER_CENTER, &format!("{remain}"), 60.0 * pulse, rgba(255, 80, 120, 1.0), true);
        // scrolling ticker of every sender in this flood
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
}

fn draw_party_disco(cx: &Cx, g: f32, t: f32) {
    let wash = hsv((t * 90.0) % 360.0, 0.65, 1.0);
    cx.rect(-20.0, -20.0, VW + 20.0, g, 0.0, Color32::from_rgba_unmultiplied(wash.r(), wash.g(), wash.b(), 40));
    for i in 0..6 {
        let a = t * 1.4 + i as f32 * TAU / 6.0;
        let (sn, cs) = a.sin_cos();
        let col = hsv((t * 120.0 + i as f32 * 60.0) % 360.0, 0.8, 1.0);
        let x = VW * 0.5 + cs * 80.0;
        cx.seg((x, 40.0), (VW * 0.5 + cs * 420.0, g - 10.0 + sn * 20.0), 18.0, Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 55));
        cx.glow(VW * 0.5 + cs * 300.0, g * 0.45, 90.0, col, 0.28);
    }
    for i in 0..16 {
        let x = 40.0 + i as f32 * 80.0;
        let pulse = 0.4 + 0.6 * ((t * 7.0 + i as f32 * 0.8).sin().abs());
        cx.glow(x, 28.0, 22.0, rgba(255, 240, 160, 1.0), pulse);
        cx.circle(x, 28.0, 5.0, rgba(255, 255, 220, pulse));
    }
}

fn current_move(moves: &[MoveKind], dance_t: f32, dance: f32) -> MoveKind {
    if moves.is_empty() {
        return MoveKind::Wave;
    }
    if moves.len() == 1 || dance <= 0.01 {
        return moves[0];
    }
    let u = clamp01(dance_t / dance);
    let i = ((u * moves.len() as f32).floor() as usize).min(moves.len() - 1);
    moves[i]
}

fn pick_challenge(rng: &mut fastrand::Rng, tier: usize, challenges: &[String], joiner: &str) -> (String, Vec<MoveKind>) {
    let list = if challenges.is_empty() { default_challenges() } else { challenges.to_vec() };
    let a = list[rng.usize(0..list.len())].clone();
    if tier >= 3 && list.len() >= 2 {
        // t4–t5: combined challenge
        let mut b = list[rng.usize(0..list.len())].clone();
        let mut guard = 0;
        while b == a && guard < 8 {
            b = list[rng.usize(0..list.len())].clone();
            guard += 1;
        }
        let title = combo_challenge(&a, &b, if joiner.is_empty() { " แล้วต่อด้วย " } else { joiner });
        let moves = parse_moves(&title, if joiner.is_empty() { " แล้วต่อด้วย " } else { joiner });
        (title, moves)
    } else {
        let moves = parse_moves(&a, joiner);
        (a, moves)
    }
}

fn pose_for(kind: MoveKind, t: f32, seed: f32, _lead: bool) -> Pose {
    let mut p = Pose::default();
    p.expr = 1;
    p.facing = 1.0;
    match kind {
        MoveKind::Robot => {
            let beat = (t * 3.0).floor();
            p.phase = beat * 0.7 + seed;
            p.walk = 0.15;
            p.hands = 0.85;
            p.raise = if (beat as i32) % 2 == 0 { 0.95 } else { 0.25 };
            p.squash = 0.06 * ((t * 6.0).sin());
            p.expr = 2;
        }
        MoveKind::SlowMo => {
            p.phase = t * 1.1 + seed;
            p.walk = 0.35;
            p.hands = 0.55;
            p.raise = 0.45 + 0.4 * (t * 0.7).sin();
            p.breath = (t * 0.8).sin();
        }
        MoveKind::Mirror => {
            p.facing = if (t * 0.7 + seed).sin() > 0.0 { 1.0 } else { -1.0 };
            p.phase = t * 4.0;
            p.hands = 0.9;
            p.raise = 0.7 + 0.25 * (t * 2.0).sin();
            p.walk = 0.2;
        }
        MoveKind::Freeze => {
            let cycle = t % 2.0;
            if cycle < 1.15 {
                p.phase = cycle * 7.0;
                p.walk = 0.45;
                p.hands = 0.7;
                p.raise = 0.55 + 0.4 * (cycle * 6.0).sin();
            } else {
                p.phase = 1.15 * 7.0;
                p.hands = 0.95;
                p.raise = 1.0;
                p.walk = 0.0;
                p.squash = 0.08;
            }
        }
        MoveKind::Chicken => {
            p.phase = t * 10.0 + seed;
            p.walk = 0.55;
            p.squash = 0.22 * (t * 10.0).sin().abs();
            p.hands = 0.4;
            p.raise = 0.15 + 0.2 * (t * 10.0).sin().abs();
            p.bow = 0.08 + 0.08 * (t * 8.0).sin().abs();
        }
        MoveKind::Fan => {
            p.hands = 1.0;
            p.raise = 0.35 + 0.55 * (t * 5.0).sin().abs();
            p.phase = t * 3.5;
            p.walk = 0.2;
        }
        MoveKind::Spin => {
            p.facing = (t * 4.5 + seed).sin();
            if p.facing.abs() < 0.08 {
                p.facing = 0.08 * p.facing.signum();
            }
            p.hands = 0.8;
            p.raise = 0.85;
            p.phase = t * 8.0;
            p.walk = 0.15;
            p.squash = -0.08 * (t * 8.0).sin().abs();
        }
        MoveKind::Jump => {
            let beat = (t * 2.4) % 1.0;
            p.squash = if beat < 0.18 {
                0.35 * (beat / 0.18)
            } else if beat < 0.5 {
                lerp(0.35, -0.55, (beat - 0.18) / 0.32)
            } else {
                lerp(-0.55, 0.05, (beat - 0.5) / 0.5)
            };
            p.hands = 0.6;
            p.raise = if beat > 0.18 && beat < 0.55 { 0.85 } else { 0.3 };
            p.phase = t * 6.0;
            p.walk = 0.1;
        }
        MoveKind::Wave => {
            p.hands = 0.95;
            p.raise = 0.55 + 0.4 * (t * 2.8 + seed).sin();
            p.phase = t * 3.0;
            p.walk = 0.15;
        }
        MoveKind::Crab => {
            p.walk = 0.7;
            p.phase = t * 7.0 + seed;
            p.hands = 0.35;
            p.squash = 0.1 * (t * 7.0).sin().abs();
            p.facing = if (t * 3.2).sin() >= 0.0 { 1.0 } else { -1.0 };
        }
    }
    p
}

fn slot(venue: Venue, i: usize, n: usize, is_lead: bool, g: f32, time: f32, s: &Show) -> (f32, f32) {
    let n = n.max(1);
    match venue {
        Venue::Floor => {
            // crowded floor: spread everyone, leave the right margin for ตัวชี้ท่า
            let span = 640.0;
            let x0 = 200.0;
            let x = if n == 1 { 560.0 } else { x0 + i as f32 * (span / (n as f32 - 1.0).max(1.0)) };
            let gy = g + if is_lead { 0.0 } else { ((i as i32 % 2) as f32) * 10.0 };
            (x.min(900.0), gy)
        }
        Venue::TempleFair => {
            let x = if is_lead { 600.0 } else { 260.0 + (i as f32 - 1.0) * 160.0 };
            (x.min(820.0), g)
        }
        Venue::Table => {
            if is_lead {
                (560.0, g - 118.0)
            } else {
                let x = if i % 2 == 1 { 260.0 } else { 820.0 };
                (x, g)
            }
        }
        Venue::Speaker => {
            let pulse = 1.0 + 0.05 * (time * 10.0).sin();
            if is_lead {
                (640.0, g - 268.0 * pulse)
            } else {
                let x = if i % 2 == 1 { 260.0 } else { 820.0 };
                (x, g)
            }
        }
        Venue::Stage => {
            if is_lead {
                (560.0, g - 42.0)
            } else {
                let x = if i % 2 == 1 { 240.0 } else { 820.0 };
                (x, g - 42.0)
            }
        }
        Venue::Parade => {
            if is_lead {
                (620.0, g - 168.0)
            } else {
                let x = 280.0 + (i as f32 - 1.0) * 140.0;
                (x.min(460.0), g)
            }
        }
        Venue::Rooftop => {
            let x = if is_lead { 600.0 } else { 260.0 + (i as f32 - 1.0) * 180.0 };
            (x.min(820.0), g - 8.0)
        }
        Venue::Foam => {
            if is_lead {
                (560.0, g)
            } else {
                let x = 220.0 + (i as f32 - 1.0) * 200.0;
                (x.min(820.0), g)
            }
        }
        Venue::Cloud => {
            let bob = (time * 1.6 + s.seed as f32 * 0.01).sin() * 10.0;
            let x = if is_lead { 560.0 } else { 260.0 + (i as f32 - 1.0) * 180.0 };
            (x.min(820.0), g - 160.0 + bob)
        }
    }
}

fn guide_slot(venue: Venue, g: f32) -> (f32, f32) {
    // keep the guide fully inside the default 540-wide "top" frame (VW=1280)
    match venue {
        Venue::Speaker => (1000.0, g),
        Venue::Table => (1000.0, g),
        Venue::Parade => (1000.0, g),
        Venue::Cloud => (1000.0, g - 150.0),
        Venue::Stage => (1000.0, g - 42.0),
        _ => (1000.0, g),
    }
}

fn draw_fan(cx: &Cx, hand: (f32, f32), t: f32) {
    let ang = -0.6 + 0.5 * (t * 8.0).sin();
    let (sn, cs) = ang.sin_cos();
    let mut pts = vec![hand];
    for i in 0..7 {
        let a = ang - 0.7 + i as f32 * 0.23;
        let (s, c) = a.sin_cos();
        pts.push((hand.0 + c * 42.0, hand.1 + s * 42.0));
    }
    // fan leaves
    for w in pts.windows(2).skip(0) {
        if w[0] == hand { continue; }
    }
    for i in 1..pts.len() {
        cx.poly(&[hand, pts[i], if i + 1 < pts.len() { pts[i + 1] } else { pts[i] }], rgba(255, 90, 110, 0.85));
    }
    cx.seg(hand, (hand.0 + cs * 38.0, hand.1 + sn * 38.0), 2.0, rgba(80, 40, 20, 0.9));
    let _ = sn;
}

fn draw_venue_sky(cx: &Cx, venue: Venue, tier: usize) {
    let h = cx.vh;
    match venue {
        Venue::Floor => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(18, 8, 40, 1.0), rgba(80, 20, 70, 1.0));
        }
        Venue::TempleFair => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(40, 30, 80, 1.0), rgba(255, 150, 80, 1.0));
        }
        Venue::Table => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(50, 20, 40, 1.0), rgba(160, 60, 70, 1.0));
        }
        Venue::Speaker => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(8, 8, 16, 1.0), rgba(40, 10, 50, 1.0));
        }
        Venue::Stage => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(10, 6, 24, 1.0), rgba(30, 10, 50, 1.0));
        }
        Venue::Parade => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(30, 50, 90, 1.0), rgba(255, 170, 70, 1.0));
        }
        Venue::Rooftop => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(8, 10, 40, 1.0), rgba(20, 24, 70, 1.0));
            // moon
            cx.glow(980.0, 90.0, 90.0, rgba(255, 240, 200, 1.0), 0.55);
            cx.circle(980.0, 90.0, 36.0, rgba(255, 245, 210, 1.0));
            cx.circle(968.0, 82.0, 12.0, rgba(230, 220, 190, 0.35));
        }
        Venue::Foam => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(20, 10, 50, 1.0), rgba(90, 30, 90, 1.0));
        }
        Venue::Cloud => {
            cx.vgradient(-40.0, -40.0, VW + 40.0, h + 40.0, rgba(30, 40, 110, 1.0), rgba(180, 140, 220, 1.0));
            cx.glow(200.0, 80.0, 120.0, rgba(255, 200, 140, 1.0), 0.4);
        }
    }
    if matches!(venue, Venue::Rooftop | Venue::Cloud | Venue::Stage | Venue::Speaker | Venue::Foam | Venue::Floor) {
        for i in 0..(18 + tier * 4) {
            let x = (i as f32 * 173.3 + cx.time * 2.0) % (VW + 20.0);
            let y = ((i as f32 * 97.1) % (h * 0.42)).abs();
            let tw = 0.4 + 0.6 * (cx.time * 2.4 + i as f32).sin();
            cx.circle(x, y, 1.6 + (i % 3) as f32, rgba(255, 255, 230, 0.25 + 0.55 * tw));
        }
    }
}

fn draw_venue_set(cx: &Cx, venue: Venue, g: f32, tier: usize, dancing: bool, bg: &str) {
    match venue {
        Venue::Floor => draw_floor(cx, g, tier, dancing),
        Venue::TempleFair => draw_temple_fair(cx, g, bg),
        Venue::Table => draw_table(cx, g),
        Venue::Speaker => draw_speakers(cx, g, dancing),
        Venue::Stage => draw_concert_stage(cx, g, tier, dancing),
        Venue::Parade => draw_parade_truck(cx, g, dancing),
        Venue::Rooftop => draw_rooftop(cx, g, bg),
        Venue::Foam => draw_foam_club(cx, g, dancing),
        Venue::Cloud => draw_cloud_stage(cx, g),
    }
    if tier >= 3 && dancing {
        draw_disco_ball(cx, 640.0, 70.0, cx.time);
    }
    if tier >= 2 && dancing && !matches!(venue, Venue::Cloud) {
        draw_spotlights(cx, g, tier, cx.time);
    }
}

fn draw_floor(cx: &Cx, g: f32, tier: usize, dancing: bool) {
    // LED tiles
    let cols = 12;
    let rows = 4;
    let tw = VW / cols as f32;
    let th = 36.0;
    for r in 0..rows {
        for c in 0..cols {
            let x0 = c as f32 * tw;
            let y0 = g - 8.0 + r as f32 * th;
            let pulse = if dancing { 0.5 + 0.5 * (cx.time * 6.0 + (c + r) as f32 * 0.7).sin() } else { 0.25 };
            let hue = ((c * 37 + r * 19) as f32 * 12.0 + cx.time * 40.0) % 360.0;
            let cfill = with_a(hsv(hue, 0.7, 0.15 + 0.7 * pulse * (0.4 + 0.15 * tier as f32)), 0.95);
            cx.rect(x0 + 2.0, y0 + 2.0, x0 + tw - 2.0, y0 + th - 2.0, 4.0, cfill);
        }
    }
}

fn draw_temple_fair(cx: &Cx, g: f32, bg: &str) {
    if bg == "scene" {
        // far chedi / roof silhouettes — festive, not a ritual parody
        let far = rgba(80, 40, 60, 0.7);
        cx.poly(&[(80.0, g - 20.0), (140.0, g - 160.0), (200.0, g - 20.0)], far);
        cx.rect(115.0, g - 200.0, 145.0, g - 150.0, 0.0, far);
        cx.poly(&[(1000.0, g - 10.0), (1100.0, g - 120.0), (1200.0, g - 10.0)], far);
    }
    // stalls
    for (x, w, col) in [(40.0, 180.0, rgba(180, 60, 50, 0.95)), (200.0, 160.0, rgba(60, 120, 80, 0.95)), (1080.0, 170.0, rgba(200, 140, 40, 0.95))] {
        cx.rect(x, g - 110.0, x + w, g, 0.0, col);
        cx.poly(&[(x - 10.0, g - 110.0), (x + w * 0.5, g - 150.0), (x + w + 10.0, g - 110.0)], rgba(140, 40, 40, 1.0));
    }
    // ธงราว
    for i in 0..16 {
        let x = 80.0 + i as f32 * 72.0;
        let y = 70.0 + (i as f32 * 0.8).sin() * 8.0;
        let hue = (i * 40) as f32;
        cx.poly(&[(x, y), (x + 22.0, y + 8.0), (x, y + 28.0)], with_a(hsv(hue, 0.8, 0.95), 0.95));
    }
    cx.seg((60.0, 64.0), (1220.0, 80.0), 2.0, rgba(40, 20, 10, 0.7));
    // hanging bulbs
    for i in 0..14 {
        let x = 100.0 + i as f32 * 80.0;
        let on = (cx.time * 6.0 + i as f32).sin() > 0.0;
        cx.circle(x, 130.0 + (i as f32).sin() * 6.0, 6.0, if on { rgba(255, 230, 120, 1.0) } else { rgba(180, 140, 60, 0.5) });
    }
    // yard
    cx.rect(-20.0, g, VW + 20.0, cx.vh + 40.0, 0.0, rgba(90, 70, 50, 1.0));
}

fn draw_table(cx: &Cx, g: f32) {
    // wooden party table
    let x0 = 400.0;
    let x1 = 880.0;
    let top = g - 118.0;
    cx.rect(x0, top, x1, top + 22.0, 4.0, rgba(150, 90, 45, 1.0));
    cx.rect(x0 + 8.0, top + 4.0, x1 - 8.0, top + 12.0, 3.0, rgba(190, 120, 60, 0.7));
    // legs
    cx.rect(x0 + 30.0, top + 22.0, x0 + 52.0, g, 3.0, rgba(90, 50, 25, 1.0));
    cx.rect(x1 - 52.0, top + 22.0, x1 - 30.0, g, 3.0, rgba(90, 50, 25, 1.0));
    // cups / bowls
    for (x, col) in [(460.0, rgba(255, 80, 80, 0.9)), (520.0, rgba(80, 180, 255, 0.9)), (800.0, rgba(255, 220, 80, 0.9))] {
        cx.ellipse(x, top - 2.0, 16.0, 8.0, col);
        cx.rect(x - 14.0, top - 18.0, x + 14.0, top, 3.0, col);
    }
    // hanging lanterns
    for (x, hue) in [(430.0, 10.0), (640.0, 40.0), (850.0, 0.0)] {
        cx.seg((x, 40.0), (x, 95.0), 2.0, rgba(40, 20, 10, 0.7));
        cx.rect(x - 18.0, 95.0, x + 18.0, 140.0, 6.0, with_a(hsv(hue, 0.8, 0.95), 0.95));
        cx.glow(x, 118.0, 30.0, hsv(hue, 0.6, 1.0), 0.35);
    }
}

fn draw_speakers(cx: &Cx, g: f32, dancing: bool) {
    let pulse = if dancing { 1.0 + 0.07 * (cx.time * 11.0).sin() } else { 1.0 };
    let cx0 = 640.0;
    // three stacked cabs + a sub
    let boxes = [(110.0, 70.0), (130.0, 80.0), (150.0, 90.0), (190.0, 70.0)];
    let mut y = g;
    for (i, (w, h)) in boxes.iter().enumerate().rev() {
        let w = *w * pulse;
        let h = *h;
        y -= h;
        cx.rect(cx0 - w * 0.5, y, cx0 + w * 0.5, y + h, 8.0, rgba(20, 16, 24, 1.0));
        cx.rect_line(cx0 - w * 0.5, y, cx0 + w * 0.5, y + h, 8.0, 3.0, rgba(60, 60, 70, 1.0));
        let r = (h * 0.32).min(w * 0.28);
        let glow_a = if dancing { 0.35 + 0.4 * (cx.time * 11.0 + i as f32).sin().abs() } else { 0.15 };
        cx.circle(cx0, y + h * 0.5, r, rgba(30, 30, 40, 1.0));
        cx.ring(cx0, y + h * 0.5, r * 0.7, 4.0, rgba(80, 80, 100, 1.0));
        cx.glow(cx0, y + h * 0.5, r * 2.2, rgba(140, 80, 255, 1.0), glow_a);
    }
    // side stacks
    for side in [-1.0, 1.0] {
        let x = cx0 + side * 260.0;
        cx.rect(x - 40.0, g - 160.0, x + 40.0, g, 6.0, rgba(18, 14, 22, 1.0));
        cx.circle(x, g - 110.0, 24.0 * pulse, rgba(40, 40, 50, 1.0));
        cx.glow(x, g - 110.0, 50.0, rgba(90, 200, 255, 1.0), if dancing { 0.3 } else { 0.1 });
    }
}

fn draw_concert_stage(cx: &Cx, g: f32, tier: usize, dancing: bool) {
    let deck = g - 42.0;
    // LED wall
    cx.rect(180.0, 40.0, 1100.0, deck - 10.0, 8.0, rgba(10, 10, 24, 1.0));
    for c in 0..14 {
        for r in 0..6 {
            let x = 200.0 + c as f32 * 64.0;
            let y = 55.0 + r as f32 * 36.0;
            let hue = (c * 25 + r * 40) as f32 + cx.time * 50.0;
            let a = if dancing { 0.35 + 0.55 * (cx.time * 4.0 + (c + r) as f32).sin().abs() } else { 0.15 };
            cx.rect(x, y, x + 52.0, y + 28.0, 3.0, with_a(hsv(hue % 360.0, 0.8, 0.9), a));
        }
    }
    // stage deck + stairs
    cx.rect(160.0, deck, 1120.0, g + 8.0, 6.0, rgba(40, 30, 50, 1.0));
    cx.rect(160.0, deck, 1120.0, deck + 10.0, 4.0, rgba(255, 200, 80, 0.7));
    for i in 0..4 {
        let y = deck + 12.0 + i as f32 * 10.0;
        cx.rect(80.0, y, 160.0, y + 8.0, 2.0, rgba(50, 40, 60, 1.0));
        cx.rect(1120.0, y, 1200.0, y + 8.0, 2.0, rgba(50, 40, 60, 1.0));
    }
    // smoke
    if dancing {
        for i in 0..6 {
            let x = 200.0 + i as f32 * 160.0 + (cx.time * 0.4 + i as f32).sin() * 20.0;
            let y = deck - 10.0 - ((cx.time * 0.6 + i as f32 * 0.7).sin().abs()) * 70.0;
            cx.ellipse(x, y, 50.0 + (i as f32 * 7.0), 22.0, rgba(200, 200, 220, 0.18));
        }
    }
    let _ = tier;
}

fn draw_parade_truck(cx: &Cx, g: f32, dancing: bool) {
    let bed = g - 168.0;
    // wheels
    for x in [260.0, 380.0, 820.0, 960.0] {
        cx.circle(x, g - 36.0, 36.0, rgba(20, 20, 24, 1.0));
        cx.circle(x, g - 36.0, 16.0, rgba(80, 80, 90, 1.0));
    }
    // chassis
    cx.rect(200.0, g - 80.0, 1040.0, g - 36.0, 4.0, rgba(40, 40, 48, 1.0));
    // cab
    cx.rect(200.0, g - 200.0, 400.0, g - 80.0, 8.0, rgba(220, 50, 50, 1.0));
    cx.rect(230.0, g - 185.0, 360.0, g - 120.0, 6.0, rgba(80, 160, 220, 0.85));
    // bed / speaker wall
    cx.rect(410.0, bed - 40.0, 1040.0, g - 80.0, 6.0, rgba(30, 20, 50, 1.0));
    cx.rect(410.0, bed, 1040.0, bed + 16.0, 4.0, rgba(255, 210, 60, 1.0));
    // LED panels
    for i in 0..8 {
        let x = 430.0 + i as f32 * 72.0;
        let hue = i as f32 * 40.0 + cx.time * 60.0;
        let a = if dancing { 0.5 + 0.5 * (cx.time * 8.0 + i as f32).sin().abs() } else { 0.25 };
        cx.rect(x, bed - 36.0, x + 64.0, bed - 4.0, 3.0, with_a(hsv(hue % 360.0, 0.85, 1.0), a));
    }
    // speaker columns on the truck
    for x in [460.0, 980.0] {
        cx.rect(x - 28.0, bed - 130.0, x + 28.0, bed, 4.0, rgba(20, 16, 24, 1.0));
        cx.circle(x, bed - 90.0, 18.0, rgba(40, 40, 50, 1.0));
        if dancing {
            cx.glow(x, bed - 90.0, 40.0, rgba(255, 80, 180, 1.0), 0.35 + 0.25 * (cx.time * 10.0).sin().abs());
        }
    }
    // pennants
    for i in 0..10 {
        let x = 420.0 + i as f32 * 60.0;
        cx.poly(&[(x, bed - 150.0), (x + 18.0, bed - 138.0), (x, bed - 126.0)], with_a(hsv(i as f32 * 36.0, 0.8, 1.0), 0.95));
    }
    cx.seg((420.0, bed - 150.0), (1020.0, bed - 150.0), 2.0, rgba(255, 255, 255, 0.8));
}

fn draw_rooftop(cx: &Cx, g: f32, bg: &str) {
    if bg == "scene" {
        // skyline
        for (x, w, h) in [(20.0, 80.0, 180.0), (110.0, 60.0, 240.0), (180.0, 100.0, 150.0), (980.0, 90.0, 200.0), (1080.0, 70.0, 260.0), (1160.0, 80.0, 140.0)] {
            cx.rect(x, g - h, x + w, g + 20.0, 0.0, rgba(12, 14, 40, 0.95));
            cx.rect(x + 8.0, g - h + 12.0, x + 20.0, g - h + 28.0, 0.0, rgba(255, 220, 120, 0.35));
        }
    }
    // roof slab + railing
    cx.rect(-20.0, g, VW + 20.0, cx.vh + 40.0, 0.0, rgba(50, 50, 62, 1.0));
    for i in 0..18 {
        let x = 40.0 + i as f32 * 70.0;
        cx.rect(x, g - 70.0, x + 6.0, g, 0.0, rgba(180, 180, 200, 0.85));
    }
    cx.rect(-20.0, g - 78.0, VW + 20.0, g - 68.0, 2.0, rgba(200, 200, 220, 0.9));
    // string lights
    for i in 0..16 {
        let x = 80.0 + i as f32 * 72.0;
        let y = 90.0 + (i as f32 * 0.7).sin() * 14.0;
        cx.circle(x, y, 5.0, rgba(255, 210, 100, 0.95));
        cx.glow(x, y, 16.0, rgba(255, 200, 80, 1.0), 0.35);
    }
    // water tank
    cx.rect(80.0, g - 160.0, 160.0, g, 6.0, rgba(70, 90, 100, 0.9));
    cx.ellipse(120.0, g - 160.0, 40.0, 12.0, rgba(90, 110, 120, 0.9));
}

fn draw_foam_club(cx: &Cx, g: f32, dancing: bool) {
    // wet floor sheen
    cx.rect(-20.0, g - 10.0, VW + 20.0, cx.vh + 40.0, 0.0, rgba(40, 20, 60, 1.0));
    for i in 0..8 {
        let x = 80.0 + i as f32 * 150.0 + (cx.time * 0.5).sin() * 10.0;
        cx.ellipse(x, g + 20.0, 80.0, 16.0, rgba(220, 230, 255, 0.18));
    }
    // foam cannons
    for x in [80.0, 1200.0] {
        cx.rect(x - 16.0, g - 80.0, x + 16.0, g, 4.0, rgba(60, 60, 80, 1.0));
        cx.rect(x - 30.0, g - 100.0, x + 30.0, g - 70.0, 8.0, rgba(80, 80, 100, 1.0));
    }
    if dancing {
        for i in 0..10 {
            let x = 100.0 + i as f32 * 110.0;
            let y = 80.0 + ((cx.time * 2.0 + i as f32).sin() * 20.0);
            cx.ellipse(x, y, 28.0, 12.0, rgba(255, 255, 255, 0.25));
        }
    }
}

fn draw_cloud_stage(cx: &Cx, g: f32) {
    let deck = g - 160.0;
    for (x, y, s) in [(200.0, deck + 30.0, 140.0), (640.0, deck + 20.0, 200.0), (1040.0, deck + 36.0, 150.0), (400.0, deck + 50.0, 110.0), (880.0, deck + 48.0, 120.0)] {
        draw_puff(cx, x, y, s);
    }
    // rainbow
    for (i, col) in [rgba(255, 80, 80, 0.45), rgba(255, 180, 40, 0.45), rgba(255, 230, 80, 0.45), rgba(80, 200, 90, 0.45), rgba(80, 140, 255, 0.45)].iter().enumerate() {
        let r = 280.0 - i as f32 * 12.0;
        // simple arc via points
        let mut pts = vec![];
        for k in 0..=16 {
            let a = PI + k as f32 / 16.0 * PI;
            pts.push((640.0 + a.cos() * r, 200.0 + a.sin() * r * 0.45));
        }
        cx.polyline(&pts, 8.0, *col);
    }
}

fn draw_puff(cx: &Cx, x: f32, y: f32, s: f32) {
    for (ox, oy, r) in [(-0.45, 0.1, 0.45), (0.0, -0.15, 0.58), (0.4, 0.08, 0.48), (0.15, 0.18, 0.4)] {
        cx.ellipse(x + ox * s, y + oy * s, r * s, r * s * 0.7, rgba(255, 255, 255, 0.85));
    }
}

fn draw_disco_ball(cx: &Cx, x: f32, y: f32, t: f32) {
    cx.seg((x, 0.0), (x, y - 26.0), 2.0, rgba(200, 200, 220, 0.7));
    cx.circle(x, y, 26.0, rgba(200, 210, 230, 1.0));
    for i in 0..10 {
        let a = t * 1.6 + i as f32 * 0.62;
        let (s, c) = a.sin_cos();
        cx.rect(x + c * 10.0 - 4.0, y + s * 10.0 - 4.0, x + c * 10.0 + 4.0, y + s * 10.0 + 4.0, 1.0, rgba(255, 255, 255, 0.7));
        cx.seg((x, y), (x + c * 180.0, y + 80.0 + s * 40.0), 2.0, rgba(255, 255, 200, 0.12));
    }
}

fn draw_spotlights(cx: &Cx, g: f32, tier: usize, t: f32) {
    let n = 2 + tier.min(4);
    for i in 0..n {
        let a = -1.1 + 0.5 * (t * (0.7 + i as f32 * 0.15) + i as f32).sin();
        let x0 = 180.0 + i as f32 * (900.0 / n.max(1) as f32);
        let x1 = x0 + a.sin() * 260.0;
        let col = hsv((i as f32 * 70.0 + t * 30.0) % 360.0, 0.6, 1.0);
        cx.poly(&[(x0 - 10.0, 20.0), (x0 + 10.0, 20.0), (x1 + 80.0, g), (x1 - 80.0, g)], with_a(col, 0.10));
        cx.glow(x1, g - 10.0, 90.0, col, 0.18);
    }
}

fn draw_crowd(cx: &Cx, g: f32, t: f32) {
    for i in 0..22 {
        let x = 30.0 + i as f32 * 58.0;
        let bob = (t * 6.0 + i as f32 * 0.6).sin().abs() * 14.0;
        let y = cx.vh - 8.0 - bob;
        cx.ellipse(x, y - 28.0, 16.0, 18.0, rgba(20, 10, 30, 0.85));
        cx.ellipse(x, y - 50.0, 12.0, 12.0, rgba(20, 10, 30, 0.85));
        let _ = g;
    }
}

pub fn appearance_for(name: &str, tier: usize) -> Look {
    let h = i_hash(name);
    Look {
        tier,
        shirt: hsv(((h >> 1) % 360) as f32, 0.55, 0.92),
        pants: hsv(((h >> 9) % 360) as f32, 0.4, 0.38),
        scale: 1.0,
        hair: hsv(((h >> 17) % 360) as f32, 0.45, 0.28),
        style: ((h >> 24) % 6) as u8,
    }
}

fn i_hash(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub fn hsv(h: f32, s: f32, v: f32) -> Color32 {
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

    fn inn(user: &str, tier: usize) -> DanceIn {
        DanceIn { user: user.into(), tier, tier_name: format!("ขั้น{}", tier + 1), gift: "Rose".into() }
    }

    #[test]
    fn venue_keys_roundtrip() {
        for v in Venue::ALL {
            assert_eq!(Venue::from_key(v.key()), Some(v));
            assert_eq!(Venue::from_key(v.label().split(" / ").next().unwrap()), Some(v), "{}", v.label());
        }
    }

    #[test]
    fn unlock_by_tier() {
        assert_eq!(Venue::Floor.min_tier(), 0);
        assert_eq!(Venue::Speaker.min_tier(), 1);
        assert_eq!(Venue::Stage.min_tier(), 2);
        assert_eq!(Venue::Foam.min_tier(), 3);
        assert_eq!(Venue::Cloud.min_tier(), 4);
        for t in 0..5 {
            let mut rng = fastrand::Rng::with_seed(t as u64 + 9);
            for _ in 0..40 {
                let v = pick_venue("auto", &[], t, &mut rng);
                assert!(v.min_tier() <= t, "t{t} unlocked {:?}", v);
            }
            let v = pick_venue("cloud", &[], 0, &mut rng);
            assert_eq!(v, Venue::Cloud, "forced venue ignores unlock");
        }
    }

    #[test]
    fn random_stays_in_enabled() {
        let en = vec!["floor".into(), "stage".into()];
        let mut rng = fastrand::Rng::with_seed(3);
        for t in 0..5 {
            let v = pick_venue("random", &en, t, &mut rng);
            assert!(v == Venue::Floor || (t >= 2 && v == Venue::Stage), "t{t} {v:?}");
        }
    }

    #[test]
    fn queue_and_floor_dancers() {
        let mut st = Stage::with_seed(1);
        let ch = default_challenges();
        st.enqueue(inn("หนึ่ง", 0), &ch, " แล้วต่อด้วย ", "floor", &[], &DEFAULT_COUNTDOWN, 6);
        st.enqueue(inn("สอง", 0), &ch, " แล้วต่อด้วย ", "floor", &[], &DEFAULT_COUNTDOWN, 6);
        st.enqueue(inn("สาม", 2), &ch, " แล้วต่อด้วย ", "floor", &[], &DEFAULT_COUNTDOWN, 6);
        assert_eq!(st.len(), 3);
        assert_eq!(st.front_user(), Some("หนึ่ง"));
        assert_eq!(st.waiting_names(), vec!["สอง", "สาม"]);
        st.tick(WALK + 0.5, 6, &DEFAULT_COUNTDOWN);
        assert!(st.lead().unwrap().in_dance());
        assert_eq!(st.shows().count(), 3, "all three stay on the crowded floor");
        for s in st.shows() {
            assert!(s.t >= s.walk || s.t > 0.0);
        }
        assert_eq!(st.active_venue(), Some(Venue::Floor));
    }

    #[test]
    fn same_sender_merges_into_combo_and_caps_time() {
        let mut st = Stage::with_seed(2);
        st.set_flood(4.0, 12.0, 99);
        let ch = default_challenges();
        for _ in 0..8 {
            st.enqueue(inn("มะลิ", 0), &ch, " แล้วต่อด้วย ", "stage", &[], &DEFAULT_COUNTDOWN, 6);
        }
        assert_eq!(st.len(), 1, "repeats must merge, not enqueue 8 cards");
        assert_eq!(st.lead().unwrap().combo, 8);
        assert!(st.lead().unwrap().dance <= 12.0 + 0.001, "extended time is capped");
        assert!(st.lead().unwrap().dance > DEFAULT_COUNTDOWN[0], "time still extends");
    }

    #[test]
    fn floor_overflow_badge_and_party_mode() {
        let mut st = Stage::with_seed(3);
        st.set_flood(4.0, 20.0, 99);
        let ch = default_challenges();
        for i in 0..12 {
            st.enqueue(inn(&format!("คน{i}"), 0), &ch, " แล้วต่อด้วย ", "floor", &[], &DEFAULT_COUNTDOWN, 8);
        }
        assert_eq!(st.shows().count(), 8, "up to max_visible dancers on the floor");
        assert!(st.waiting_extra() >= 4, "the rest wait behind a +N badge");
        assert!(!st.in_party());

        let mut st = Stage::with_seed(4);
        st.set_flood(4.0, 20.0, 6);
        for i in 0..16 {
            st.enqueue(inn(&format!("แฟน{i}"), 1), &ch, " แล้วต่อด้วย ", "stage", &[], &DEFAULT_COUNTDOWN, 8);
        }
        assert!(st.in_party(), "crossing the rate threshold enters โหมดปาร์ตี้");
        assert!(st.shows().count() <= 8);
        assert!(st.live_units() <= 200, "flood records stay bounded, got {}", st.live_units());
        // drain: after the party timer, leftover waiters are cleared and dancers leave
        st.tick(21.0, 8, &DEFAULT_COUNTDOWN);
        assert!(!st.in_party());
        st.tick(WALK + 2.0, 8, &DEFAULT_COUNTDOWN);
        assert!(st.waiting_extra() == 0);
        assert!(st.live_units() < 80);
    }

    #[test]
    fn combo_at_high_tier() {
        let mut rng = fastrand::Rng::with_seed(11);
        let ch = default_challenges();
        let (title, moves) = pick_challenge(&mut rng, 4, &ch, " แล้วต่อด้วย ");
        assert!(title.contains("แล้วต่อด้วย"), "{title}");
        assert!(moves.len() >= 2, "{moves:?}");
    }

    #[test]
    fn challenges_are_thai_and_safe() {
        for c in default_challenges() {
            assert!(c.chars().any(|ch| (ch as u32) >= 0x0E00 && (ch as u32) <= 0x0E7F), "{c}");
            let low = c.to_lowercase();
            for bad in ["sexy", "nude", "18+", "kill", "ยา"] {
                assert!(!low.contains(bad), "{c}");
            }
        }
    }

    #[test]
    fn countdown_scales() {
        assert_eq!(countdown_for(&[], 0), 8.0);
        assert_eq!(countdown_for(&[], 4), 24.0);
        assert_eq!(countdown_for(&[1.0, 2.0, 3.0, 4.0, 5.0], 2), 3.0);
    }

    #[test]
    fn lead_finishes_and_promotes() {
        let mut st = Stage::with_seed(2);
        let ch = default_challenges();
        st.enqueue(inn("เอ", 0), &ch, " แล้วต่อด้วย ", "stage", &[], &[2.0, 2.0, 2.0, 2.0, 2.0], 6);
        st.enqueue(inn("บี", 1), &ch, " แล้วต่อด้วย ", "stage", &[], &[2.0, 2.0, 2.0, 2.0, 2.0], 6);
        st.tick(WALK + 2.0 + LEAVE + 0.05, 6, &[2.0, 2.0, 2.0, 2.0, 2.0]);
        assert_eq!(st.front_user(), Some("บี"));
        assert!(st.lead().unwrap().in_dance() || st.lead().unwrap().t >= st.lead().unwrap().walk);
    }

    #[test]
    fn parse_each_default_move() {
        for c in default_challenges() {
            let m = parse_moves(&c, " แล้วต่อด้วย ");
            assert_eq!(m.len(), 1, "{c} -> {m:?}");
        }
    }
}
