//! โหมดเลี้ยงแมว: แมวถาวรตัวหนึ่ง ผู้ชมช่วยกันเลี้ยงผ่านของขวัญ
//! ลูบหัว แปะอุ้งเท้า ไล่ไม้ขนกล่อ ให้อาหาร — มีแถบความเหงา / ความอิ่ม / ความผูกพัน (+เลเวล)
//! สีหน้าและท่าทางของแมวเปลี่ยนตามแถบ ไม่มีเลขมงคล
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::{BTreeMap, VecDeque};
use std::f32::consts::{PI, TAU};

const CREAM: Color32 = Color32::from_rgb(255, 244, 224);
const INK: Color32 = Color32::from_rgb(48, 34, 40);
const PINK: Color32 = Color32::from_rgb(255, 150, 175);
const GOLD: Color32 = Color32::from_rgb(255, 208, 96);
const HEART: Color32 = Color32::from_rgb(255, 105, 150);

pub const PET_X: f32 = 540.0;
pub const BOWL_X: f32 = 800.0;
pub const BED_X: f32 = 250.0;

pub fn floor_y(vh: f32) -> f32 {
    (vh - 96.0_f32.min(vh * 0.14)).max(vh * 0.62)
}

/// ของขวัญ -> กิมมิก ไล่ตามราคา (10 อย่าง) ค่าเริ่มต้นจับคู่สติกเกอร์จริงใน catalog
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trick {
    HeadPet,
    PawFive,
    WandChase,
    DonutTreat,
    MilkBowl,
    FishToy,
    FriendVisit,
    Knead,
    MilkBottle,
    GoldenKitten,
}

pub const TRICKS: [Trick; 10] = [
    Trick::HeadPet,
    Trick::PawFive,
    Trick::WandChase,
    Trick::DonutTreat,
    Trick::MilkBowl,
    Trick::FishToy,
    Trick::FriendVisit,
    Trick::Knead,
    Trick::MilkBottle,
    Trick::GoldenKitten,
];

impl Trick {
    pub fn key(self) -> &'static str {
        match self {
            Trick::HeadPet => "head_pet",
            Trick::PawFive => "paw_five",
            Trick::WandChase => "wand_chase",
            Trick::DonutTreat => "donut_treat",
            Trick::MilkBowl => "milk_bowl",
            Trick::FishToy => "fish_toy",
            Trick::FriendVisit => "friend_visit",
            Trick::Knead => "knead",
            Trick::MilkBottle => "milk_bottle",
            Trick::GoldenKitten => "golden_kitten",
        }
    }
    pub fn from_key(s: &str) -> Option<Trick> {
        let s = s.trim();
        TRICKS.iter().copied().find(|t| t.key().eq_ignore_ascii_case(s))
    }
    pub fn label(self) -> &'static str {
        match self {
            Trick::HeadPet => "ลูบหัว",
            Trick::PawFive => "แปะอุ้งเท้า",
            Trick::WandChase => "ไล่ไม้ขนกล่อ",
            Trick::DonutTreat => "ให้โดนัทแมว",
            Trick::MilkBowl => "นมชามใหญ่",
            Trick::FishToy => "ปลาของเล่น",
            Trick::FriendVisit => "เพื่อนแมวมาเล่น",
            Trick::Knead => "แมวนวดทำบิสกิต",
            Trick::MilkBottle => "นมขวดจุกใหญ่",
            Trick::GoldenKitten => "ลูกแมวทองคำ",
        }
    }
    /// สติกเกอร์เริ่มต้นของกิมมิกนี้ (ใช้ตอนไม่มีของ map ไว้)
    pub fn gift(self) -> &'static str {
        match self {
            Trick::HeadPet => "Rose",
            Trick::PawFive => "Brown Paw",
            Trick::WandChase => "Magic Feather",
            Trick::DonutTreat => "Doughnut",
            Trick::MilkBowl => "Sundae Bowl",
            Trick::FishToy => "Puffer Fish",
            Trick::FriendVisit => "Fan Cat",
            Trick::Knead => "Kitten Kneading",
            Trick::MilkBottle => "Reindeer Milk",
            Trick::GoldenKitten => "Leon the Kitten",
        }
    }
    pub fn diamonds(self) -> u64 {
        match self {
            Trick::HeadPet => 1,
            Trick::PawFive => 5,
            Trick::WandChase => 10,
            Trick::DonutTreat => 30,
            Trick::MilkBowl => 99,
            Trick::FishToy => 150,
            Trick::FriendVisit => 199,
            Trick::Knead => 399,
            Trick::MilkBottle => 400,
            Trick::GoldenKitten => 4888,
        }
    }
    pub fn seconds(self) -> f32 {
        match self {
            Trick::HeadPet => 2.8,
            Trick::PawFive => 2.6,
            Trick::WandChase => 4.6,
            Trick::DonutTreat => 3.2,
            Trick::MilkBowl => 3.6,
            Trick::FishToy => 4.2,
            Trick::FriendVisit => 5.6,
            Trick::Knead => 5.0,
            Trick::MilkBottle => 4.0,
            Trick::GoldenKitten => 7.0,
        }
    }
    /// (เหงาที่ลด, อิ่มที่เพิ่ม, ผูกพันที่เพิ่ม) — ของเล่นเผาผลาญพลังงานจึงหิวเร็วขึ้นเล็กน้อย
    pub fn effect(self) -> (f32, f32, f32) {
        match self {
            Trick::HeadPet => (18.0, 0.0, 3.0),
            Trick::PawFive => (14.0, 0.0, 4.0),
            Trick::WandChase => (30.0, -6.0, 6.0),
            Trick::DonutTreat => (6.0, 22.0, 4.0),
            Trick::MilkBowl => (8.0, 40.0, 6.0),
            Trick::FishToy => (22.0, 14.0, 7.0),
            Trick::FriendVisit => (45.0, -4.0, 10.0),
            Trick::Knead => (26.0, 0.0, 16.0),
            Trick::MilkBottle => (10.0, 70.0, 9.0),
            Trick::GoldenKitten => (100.0, 100.0, 40.0),
        }
    }
    pub fn is_food(self) -> bool {
        matches!(self, Trick::DonutTreat | Trick::MilkBowl | Trick::FishToy | Trick::MilkBottle | Trick::GoldenKitten)
    }
    pub fn is_star(self) -> bool {
        self == Trick::GoldenKitten
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mood {
    Playful,
    Lonely,
    Hungry,
    Happy,
    Content,
}

impl Mood {
    pub fn key(self) -> &'static str {
        match self {
            Mood::Playful => "playful",
            Mood::Lonely => "lonely",
            Mood::Hungry => "hungry",
            Mood::Happy => "happy",
            Mood::Content => "content",
        }
    }
    pub fn bubble(self) -> &'static str {
        match self {
            Mood::Playful => "สนุกจังเลย~",
            Mood::Lonely => "เหงาจัง… มาเล่นกันหน่อย",
            Mood::Hungry => "หิวแล้วน้า ขอของกินหน่อย",
            Mood::Happy => "ฟินมาก ขอบคุณน้า~",
            Mood::Content => "อยู่เป็นเพื่อนนะ",
        }
    }
}

pub fn level_title(level: usize) -> &'static str {
    match level {
        1 => "ลูกแมวแว๋ว",
        2 => "แมวขี้อ้อน",
        3 => "แมวเพื่อนซี้",
        4 => "แมวเจ้าสำราญ",
        _ => "แมวทองคำ",
    }
}

/// ของขวัญ -> กิมมิก: ตารางที่ตั้งไว้ > บันไดราคาในตัว > กิมมิกแพงสุดที่เพชรถึง
pub fn trick_for(gift: &str, gift_id: u64, diamonds: u64, overrides: &BTreeMap<String, String>) -> Trick {
    let name = gift.trim();
    for (k, v) in overrides {
        let k = k.trim();
        if (!name.is_empty() && k.eq_ignore_ascii_case(name)) || (gift_id != 0 && k == gift_id.to_string()) {
            if let Some(t) = Trick::from_key(v) {
                return t;
            }
        }
    }
    if let Some(t) = TRICKS.iter().copied().find(|t| t.gift().eq_ignore_ascii_case(name)) {
        return t;
    }
    let mut best = Trick::HeadPet;
    for t in TRICKS {
        if t.diamonds() <= diamonds.max(1) {
            best = t;
        }
    }
    best
}

pub struct PetIn {
    pub user: String,
    pub tier: usize,
    pub gift: String,
    pub gift_id: u64,
    pub diamonds: u64,
    pub trick: Trick,
}

pub struct Interaction {
    pub trick: Trick,
    pub user: String,
    pub tier: usize,
    pub gift: String,
    pub gift_id: u64,
    pub diamonds: u64,
    pub t: f32,
    pub dur: f32,
    pub combo: u32,
    pub seed: f32,
    at: f32,
}

impl Interaction {
    pub fn phase(&self) -> &'static str {
        let u = self.t / self.dur.max(0.01);
        if u < 0.16 {
            "reach"
        } else if u < 0.78 {
            "play"
        } else {
            "thanks"
        }
    }
    pub fn progress(&self) -> f32 {
        (self.t / self.dur.max(0.01)).clamp(0.0, 1.0)
    }
}

pub struct Pending {
    pub user: String,
    pub tier: usize,
    pub gift: String,
    pub gift_id: u64,
    pub diamonds: u64,
    pub trick: Trick,
    pub combo: u32,
    at: f32,
}

struct Heart {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    size: f32,
    spin: f32,
}

struct Look {
    fur: Color32,
    fur_dark: Color32,
    fur_light: Color32,
    belly: Color32,
    scale: f32,
    seed: f32,
    stripes: bool,
    level: usize,
}

fn look_for(level: usize, seed: f32) -> Look {
    let base = match level {
        1 => (Color32::from_rgb(250, 214, 170), Color32::from_rgb(226, 178, 128)),
        2 => (Color32::from_rgb(252, 196, 140), Color32::from_rgb(226, 158, 100)),
        3 => (Color32::from_rgb(250, 182, 120), Color32::from_rgb(220, 142, 86)),
        4 => (Color32::from_rgb(246, 170, 106), Color32::from_rgb(212, 128, 72)),
        _ => (Color32::from_rgb(255, 214, 120), Color32::from_rgb(226, 168, 62)),
    };
    Look {
        fur: base.0,
        fur_dark: base.1,
        fur_light: shade(base.0, 1.14),
        belly: CREAM,
        scale: 1.15 + (level.min(5) as f32) * 0.06,
        seed,
        level,
        stripes: level >= 2,
    }
}

struct Pose {
    mood: Mood,
    blink: f32,
    ear: f32,
    hop: f32,
    lean: f32,
    pupil: f32,
    smile: f32,
    play: f32,
    trick: Option<Trick>,
    u: f32,
    tail: f32,
}

pub struct PetStage {
    /// 0..100 — สูง = เหงา (เพิ่มเมื่อไม่มีคนเล่นนาน)
    pub loneliness: f32,
    /// 0..100 — สูง = อิ่ม (ลดตามเวลา)
    pub fullness: f32,
    /// 0..100 ภายในเลเวลปัจจุบัน
    pub bond: f32,
    pub level: usize,
    pub name: String,
    pub started: u64,
    pub finished: u64,
    clock: f32,
    since_touch: f32,
    active: Option<Interaction>,
    queue: VecDeque<Pending>,
    overflow_extra: u64,
    hearts: Vec<Heart>,
    recent: VecDeque<String>,
    level_flash: f32,
    seed: f32,
    rng: fastrand::Rng,
    lonely_after: f32,
    lonely_rate: f32,
    hungry_rate: f32,
    max_level: usize,
    max_queue: usize,
    merge_window: f32,
    max_seconds: f32,
}

impl PetStage {
    pub fn new() -> Self {
        Self::with_seed(0x9E3779B97F4A7C15)
    }
    pub fn with_seed(seed: u64) -> Self {
        let mut rng = fastrand::Rng::with_seed(seed);
        let s = rng.f32();
        Self {
            loneliness: 12.0,
            fullness: 78.0,
            bond: 0.0,
            level: 1,
            name: "น้องมอมแมม".into(),
            started: 0,
            finished: 0,
            clock: 0.0,
            since_touch: 0.0,
            active: None,
            queue: VecDeque::new(),
            overflow_extra: 0,
            hearts: vec![],
            recent: VecDeque::new(),
            level_flash: 0.0,
            seed: s,
            rng,
            lonely_after: 20.0,
            lonely_rate: 1.6,
            hungry_rate: 0.9,
            max_level: 5,
            max_queue: 30,
            merge_window: 4.0,
            max_seconds: 14.0,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_tuning(&mut self, name: &str, lonely_after: f32, lonely_rate: f32, hungry_rate: f32, max_level: usize, max_queue: usize, merge_window: f32, max_seconds: f32) {
        if !name.trim().is_empty() {
            self.name = name.trim().to_string();
        }
        self.lonely_after = lonely_after.clamp(2.0, 600.0);
        self.lonely_rate = lonely_rate.clamp(0.0, 40.0);
        self.hungry_rate = hungry_rate.clamp(0.0, 40.0);
        self.max_level = max_level.clamp(1, 9);
        self.max_queue = max_queue.clamp(1, 200);
        self.merge_window = merge_window.clamp(0.0, 30.0);
        self.max_seconds = max_seconds.clamp(2.0, 90.0);
        self.level = self.level.min(self.max_level);
    }

    pub fn mood(&self) -> Mood {
        if self.active.is_some() {
            return Mood::Playful;
        }
        if self.loneliness >= 70.0 {
            Mood::Lonely
        } else if self.fullness <= 25.0 {
            Mood::Hungry
        } else if self.loneliness <= 25.0 && self.fullness >= 60.0 {
            Mood::Happy
        } else {
            Mood::Content
        }
    }

    pub fn lead(&self) -> Option<&Interaction> {
        self.active.as_ref()
    }
    pub fn queued(&self) -> &VecDeque<Pending> {
        &self.queue
    }
    pub fn len(&self) -> usize {
        self.queue.len() + if self.active.is_some() { 1 } else { 0 }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn busy(&self) -> bool {
        self.active.is_some() || !self.queue.is_empty()
    }
    pub fn combo(&self) -> u32 {
        self.active.as_ref().map(|a| a.combo).unwrap_or(1)
    }
    pub fn waiting_extra(&self) -> usize {
        self.queue.len() + self.overflow_extra as usize
    }
    pub fn live_units(&self) -> usize {
        self.len() + self.hearts.len() + self.recent.len()
    }
    pub fn overflow_extra(&self) -> u64 {
        self.overflow_extra
    }
    pub fn level_flash(&self) -> f32 {
        self.level_flash
    }
    pub fn since_touch(&self) -> f32 {
        self.since_touch
    }
    pub fn lonely_after(&self) -> f32 {
        self.lonely_after
    }

    /// ล้างคิว/กิมมิกที่กำลังเล่น แต่แมวและแถบสถานะยังอยู่ (แมวถาวร)
    pub fn clear(&mut self) {
        self.active = None;
        self.queue.clear();
        self.overflow_extra = 0;
        self.hearts.clear();
        self.recent.clear();
    }

    /// เริ่มเลี้ยงใหม่ทั้งหมด (ปุ่มในหน้าตั้งค่า)
    pub fn reset_stats(&mut self) {
        self.clear();
        self.loneliness = 12.0;
        self.fullness = 78.0;
        self.bond = 0.0;
        self.level = 1;
        self.since_touch = 0.0;
        self.started = 0;
        self.finished = 0;
        self.level_flash = 0.0;
    }

    pub fn enqueue(&mut self, inn: PetIn) {
        let (clock, window) = (self.clock, self.merge_window);
        let same = |u: &str, t: Trick, at: f32| u == inn.user && t == inn.trick && clock - at <= window;
        let merged_active = match &mut self.active {
            Some(a) if same(&a.user, a.trick, a.at) && a.t < a.dur * 0.7 => {
                a.combo = a.combo.saturating_add(1);
                a.dur = (a.dur + 0.5).min(self.max_seconds);
                true
            }
            _ => false,
        };
        if merged_active {
            self.touch_recent(&inn.user);
            return;
        }
        let merged_back = match self.queue.back_mut() {
            Some(back) if same(&back.user, back.trick, back.at) => {
                back.combo = back.combo.saturating_add(1);
                true
            }
            _ => false,
        };
        if merged_back {
            self.touch_recent(&inn.user);
            return;
        }
        if self.queue.len() >= self.max_queue {
            self.overflow_extra = self.overflow_extra.saturating_add(1);
            return;
        }
        let p = Pending { user: inn.user, tier: inn.tier, gift: inn.gift, gift_id: inn.gift_id, diamonds: inn.diamonds, trick: inn.trick, combo: 1, at: self.clock };
        self.queue.push_back(p);
        if self.active.is_none() {
            if let Some(p) = self.queue.pop_front() {
                self.begin(p);
            }
        }
    }

    fn touch_recent(&mut self, user: &str) {
        self.recent.retain(|u| u != user);
        self.recent.push_front(user.to_string());
        while self.recent.len() > 12 {
            self.recent.pop_back();
        }
    }

    fn begin(&mut self, p: Pending) {
        let (dl, df, db) = p.trick.effect();
        self.loneliness = (self.loneliness - dl).clamp(0.0, 100.0);
        self.fullness = (self.fullness + df).clamp(0.0, 100.0);
        self.add_bond(db);
        self.since_touch = 0.0;
        let dur = (p.trick.seconds() + (p.combo.saturating_sub(1) as f32) * 0.4).min(self.max_seconds);
        let seed = self.rng.f32();
        self.active = Some(Interaction {
            trick: p.trick,
            user: p.user.clone(),
            tier: p.tier,
            gift: p.gift,
            gift_id: p.gift_id,
            diamonds: p.diamonds,
            t: 0.0,
            dur,
            combo: p.combo.max(1),
            seed,
            at: self.clock,
        });
        self.started += 1;
        self.touch_recent(&p.user);
        let n = if p.trick.is_star() { 26 } else { 8 + (p.combo.min(6) as usize) };
        for _ in 0..n {
            self.spawn_heart(p.trick.is_star());
        }
    }

    fn add_bond(&mut self, amount: f32) {
        self.bond += amount;
        while self.bond >= 100.0 && self.level < self.max_level {
            self.bond -= 100.0;
            self.level += 1;
            self.level_flash = 2.2;
            for _ in 0..18 {
                self.spawn_heart(true);
            }
        }
        if self.level >= self.max_level {
            self.bond = self.bond.min(100.0);
        }
    }

    fn spawn_heart(&mut self, big: bool) {
        if self.hearts.len() > 160 {
            return;
        }
        let a = self.rng.f32() * TAU;
        self.hearts.push(Heart {
            x: PET_X + (self.rng.f32() - 0.5) * 150.0,
            y: 0.0,
            vx: a.cos() * 26.0,
            vy: -46.0 - self.rng.f32() * 54.0,
            life: 0.0,
            max: if big { 2.4 } else { 1.7 },
            size: if big { 13.0 + self.rng.f32() * 12.0 } else { 8.0 + self.rng.f32() * 8.0 },
            spin: (self.rng.f32() - 0.5) * 1.6,
        });
    }

    pub fn tick(&mut self, dt: f32) {
        self.clock += dt;
        if self.level_flash > 0.0 {
            self.level_flash = (self.level_flash - dt).max(0.0);
        }
        if self.active.is_some() {
            self.since_touch = 0.0;
        } else {
            self.since_touch += dt;
            if self.since_touch > self.lonely_after {
                self.loneliness = (self.loneliness + self.lonely_rate * dt).min(100.0);
            }
        }
        self.fullness = (self.fullness - self.hungry_rate * dt).max(0.0);

        if let Some(a) = &mut self.active {
            a.t += dt;
        }
        if self.active.as_ref().map(|a| a.t >= a.dur).unwrap_or(false) {
            self.active = None;
            self.finished += 1;
        }
        if self.active.is_none() {
            if let Some(p) = self.queue.pop_front() {
                self.begin(p);
            }
        }

        for h in &mut self.hearts {
            h.life += dt;
            h.x += h.vx * dt;
            h.y += h.vy * dt;
            h.vy += 12.0 * dt;
            h.vx *= (-0.7 * dt).exp();
        }
        self.hearts.retain(|h| h.life < h.max);
    }

    fn pose_of(&self) -> Pose {
        let mood = self.mood();
        let bt = (self.clock * 0.55 + self.seed * 7.0) % 3.6;
        let blink = if bt < 0.13 { (bt / 0.13 * PI).sin() } else { 0.0 };
        let a = self.active.as_ref();
        let u = a.map(|x| x.progress()).unwrap_or(0.0);
        let trick = a.map(|x| x.trick);
        let play = if a.is_some() { (u * PI).sin().clamp(0.0, 1.0) } else { 0.0 };
        let chasing = matches!(trick, Some(Trick::WandChase) | Some(Trick::FishToy) | Some(Trick::FriendVisit));
        let hop = if chasing { (self.clock * 7.5).sin().abs() * 26.0 * play } else { play * 6.0 };
        let lean = if chasing { (self.clock * 3.4).sin() * 0.16 * play } else { (self.clock * 0.9 + self.seed).sin() * 0.03 };
        let (ear, pupil, smile) = match mood {
            Mood::Lonely => (-0.85, 0.55, -0.5),
            Mood::Hungry => (-0.25, 0.8, -0.15),
            Mood::Playful => (1.0, 1.25, 0.9),
            Mood::Happy => (0.55, 0.7, 1.0),
            Mood::Content => (0.25, 0.85, 0.45),
        };
        Pose {
            mood,
            blink: if matches!(mood, Mood::Happy) { 0.85 } else { blink },
            ear,
            hop,
            lean,
            pupil,
            smile,
            play,
            trick,
            u,
            tail: self.clock * if matches!(mood, Mood::Playful | Mood::Happy) { 7.0 } else { 1.9 } + self.seed * 5.0,
        }
    }

    // ------------------------------------------------------------------ world

    pub fn draw_world(&self, cx: &Cx, props: &Cx, bg: &str) {
        let g = floor_y(cx.vh);
        let pose = self.pose_of();
        let look = look_for(self.level, self.seed);
        if bg == "scene" {
            self.draw_room(cx, g);
        }
        self.draw_props(props, g);
        if cx.halo {
            cx.halo(PET_X, g - 70.0 * look.scale, 190.0 * look.scale, 120.0 * look.scale);
        }
        self.draw_friend(cx, g, &pose);
        draw_pet(cx, PET_X, g, &look, &pose, 1.0);
        self.draw_trick_prop(cx, g, &pose, &look);
        self.draw_hearts(cx, g);
    }

    fn draw_room(&self, cx: &Cx, g: f32) {
        let h = cx.vh;
        cx.vgradient(-40.0, -40.0, VW + 40.0, g, Color32::from_rgb(255, 232, 214), Color32::from_rgb(250, 206, 186));
        cx.rect(-40.0, g, VW + 40.0, h + 40.0, 0.0, Color32::from_rgb(206, 160, 124));
        for i in 0..9 {
            let y = g + 12.0 + i as f32 * 17.0;
            cx.line((-40.0, y), (VW + 40.0, y), 2.0, rgba(170, 126, 96, 90.0 / 255.0));
        }
        // window with a slow day cycle (left wall, clear of the meters card)
        let (wx, wy, ww, wh) = (70.0, h * 0.33, 250.0, 200.0);
        cx.rect(wx - 10.0, wy - 10.0, wx + ww + 10.0, wy + wh + 10.0, 18.0, Color32::from_rgb(255, 250, 240));
        let night = (cx.time * 0.05).sin() > 0.0;
        let (top, bot) = if night {
            (Color32::from_rgb(38, 44, 96), Color32::from_rgb(86, 74, 140))
        } else {
            (Color32::from_rgb(150, 214, 255), Color32::from_rgb(224, 244, 255))
        };
        cx.vgradient(wx, wy, wx + ww, wy + wh, top, bot);
        if night {
            cx.circle(wx + ww * 0.72, wy + wh * 0.3, 22.0, Color32::from_rgb(255, 248, 214));
            cx.circle(wx + ww * 0.66, wy + wh * 0.26, 19.0, top);
            for i in 0..12 {
                let sx = wx + ((i as f32 * 53.7) % ww);
                let sy = wy + ((i as f32 * 37.3) % wh);
                let tw = 0.4 + 0.6 * (cx.time * 2.0 + i as f32).sin().abs();
                cx.circle(sx, sy, 1.8, rgba(255, 255, 235, (200.0 * tw) / 255.0));
            }
        } else {
            cx.circle(wx + ww * 0.28, wy + wh * 0.28, 24.0, Color32::from_rgb(255, 236, 150));
            for (ox, oy, r) in [(0.55, 0.55, 20.0), (0.72, 0.5, 26.0), (0.86, 0.58, 18.0)] {
                cx.circle(wx + ww * ox, wy + wh * oy, r, rgba(255, 255, 255, 225.0 / 255.0));
            }
        }
        cx.line((wx + ww * 0.5, wy), (wx + ww * 0.5, wy + wh), 7.0, Color32::from_rgb(255, 250, 240));
        cx.line((wx, wy + wh * 0.5), (wx + ww, wy + wh * 0.5), 7.0, Color32::from_rgb(255, 250, 240));
        // framed paw print (right wall, below the guide card)
        let (fx, fy) = (880.0, h * 0.33);
        cx.rect(fx, fy, fx + 120.0, fy + 104.0, 10.0, Color32::from_rgb(255, 252, 244));
        cx.rect_line(fx, fy, fx + 120.0, fy + 104.0, 10.0, 6.0, Color32::from_rgb(214, 164, 120));
        draw_paw(cx, fx + 60.0, fy + 58.0, 22.0, Color32::from_rgb(240, 170, 150));
        // wall shelf with a plant and books
        let (sx, sy) = (380.0, h * 0.40);
        cx.rect(sx, sy, sx + 240.0, sy + 12.0, 4.0, Color32::from_rgb(214, 164, 120));
        cx.rect(sx + 18.0, sy - 34.0, sx + 52.0, sy, 5.0, Color32::from_rgb(226, 140, 120));
        cx.circle(sx + 35.0, sy - 44.0, 16.0, Color32::from_rgb(140, 200, 140));
        cx.circle(sx + 24.0, sy - 52.0, 11.0, Color32::from_rgb(160, 214, 150));
        cx.circle(sx + 46.0, sy - 52.0, 11.0, Color32::from_rgb(120, 186, 130));
        for (i, c) in [Color32::from_rgb(150, 190, 230), Color32::from_rgb(250, 200, 140), Color32::from_rgb(230, 150, 170)].iter().enumerate() {
            cx.rect(sx + 80.0 + i as f32 * 22.0, sy - 40.0, sx + 96.0 + i as f32 * 22.0, sy, 3.0, *c);
        }
        cx.rect(sx + 160.0, sy - 26.0, sx + 210.0, sy, 6.0, Color32::from_rgb(255, 250, 240));
        cx.circle(sx + 185.0, sy - 26.0, 12.0, Color32::from_rgb(255, 178, 196));
        // round wall clock
        let (kx, ky) = (720.0, h * 0.40);
        cx.circle(kx, ky, 40.0, Color32::from_rgb(255, 252, 244));
        cx.ring(kx, ky, 40.0, 5.0, Color32::from_rgb(214, 164, 120));
        let ha = cx.time * 0.05;
        cx.line((kx, ky), (kx + ha.cos() * 20.0, ky + ha.sin() * 20.0), 4.0, INK);
        cx.line((kx, ky), (kx + (ha * 12.0).cos() * 28.0, ky + (ha * 12.0).sin() * 28.0), 3.0, INK);
        cx.circle(kx, ky, 4.0, Color32::from_rgb(226, 140, 120));
        // fairy-light garland across the wall
        let gy = h * 0.56;
        let mut pts = Vec::new();
        for i in 0..=24 {
            let t = i as f32 / 24.0;
            pts.push((40.0 + t * (VW - 80.0), gy + (t * 3.0 * PI).sin().abs() * 26.0));
        }
        cx.polyline(&pts, 2.5, rgba(180, 140, 120, 0.5));
        for i in 0..12 {
            let t = (i as f32 + 0.5) / 12.0;
            let x = 40.0 + t * (VW - 80.0);
            let sag = (t * 3.0 * PI).sin().abs() * 26.0;
            let c = [Color32::from_rgb(255, 170, 190), Color32::from_rgb(255, 220, 150), Color32::from_rgb(170, 220, 255)][i % 3];
            let tw = 0.6 + 0.4 * (cx.time * 2.0 + i as f32).sin().abs();
            cx.circle(x, gy + sag + 8.0, 5.0, with_a(c, tw));
        }
        // rug
        cx.ellipse(PET_X, g + 16.0, 300.0, 46.0, Color32::from_rgb(244, 178, 168));
        cx.ellipse(PET_X, g + 16.0, 250.0, 36.0, Color32::from_rgb(250, 200, 188));
        cx.ellipse(PET_X, g + 16.0, 190.0, 26.0, Color32::from_rgb(252, 216, 204));
    }

    fn draw_props(&self, cx: &Cx, g: f32) {
        // cat bed
        let bx = BED_X;
        cx.ellipse(bx, g + 6.0, 104.0, 22.0, rgba(0, 0, 0, 40.0 / 255.0));
        cx.rect(bx - 96.0, g - 40.0, bx + 96.0, g + 8.0, 22.0, Color32::from_rgb(196, 140, 176));
        cx.ellipse(bx, g - 34.0, 92.0, 24.0, Color32::from_rgb(226, 176, 206));
        cx.ellipse(bx, g - 30.0, 74.0, 17.0, Color32::from_rgb(248, 214, 232));
        // food bowl: fill follows fullness
        let f = self.fullness / 100.0;
        cx.ellipse(BOWL_X, g + 4.0, 56.0, 12.0, rgba(0, 0, 0, 40.0 / 255.0));
        cx.poly(
            &[(BOWL_X - 52.0, g - 30.0), (BOWL_X + 52.0, g - 30.0), (BOWL_X + 38.0, g), (BOWL_X - 38.0, g)],
            Color32::from_rgb(120, 190, 220),
        );
        cx.ellipse(BOWL_X, g - 30.0, 52.0, 13.0, Color32::from_rgb(160, 214, 238));
        if f > 0.02 {
            cx.ellipse(BOWL_X, g - 30.0, 44.0 * f.max(0.25), 10.0 * f.max(0.25), Color32::from_rgb(255, 248, 232));
            for i in 0..5 {
                let k = i as f32;
                cx.circle(BOWL_X - 26.0 + k * 13.0, g - 32.0 + (k * 1.7).sin() * 3.0, 5.0, Color32::from_rgb(214, 164, 110));
            }
        }
        draw_paw(cx, BOWL_X, g - 12.0, 11.0, rgba(255, 255, 255, 190.0 / 255.0));
        // toy basket
        let tx = 960.0;
        cx.rect(tx - 46.0, g - 44.0, tx + 46.0, g, 10.0, Color32::from_rgb(214, 170, 116));
        for i in 0..4 {
            cx.line((tx - 46.0, g - 36.0 + i as f32 * 10.0), (tx + 46.0, g - 36.0 + i as f32 * 10.0), 2.5, rgba(170, 126, 80, 150.0 / 255.0));
        }
        cx.circle(tx - 16.0, g - 50.0, 15.0, Color32::from_rgb(255, 138, 168));
        cx.circle(tx + 14.0, g - 54.0, 13.0, Color32::from_rgb(150, 214, 255));
        cx.ring(tx + 14.0, g - 54.0, 13.0, 3.0, Color32::from_rgb(255, 255, 255));
        // scratching post
        let sx = 90.0;
        cx.rect(sx - 26.0, g - 6.0, sx + 26.0, g + 4.0, 6.0, Color32::from_rgb(190, 146, 104));
        cx.rect(sx - 11.0, g - 150.0, sx + 11.0, g - 4.0, 5.0, Color32::from_rgb(226, 196, 150));
        for i in 0..12 {
            let y = g - 146.0 + i as f32 * 12.0;
            cx.line((sx - 11.0, y), (sx + 11.0, y + 4.0), 2.0, rgba(180, 146, 100, 160.0 / 255.0));
        }
        cx.circle(sx, g - 158.0, 17.0, Color32::from_rgb(255, 178, 196));
    }

    fn draw_friend(&self, cx: &Cx, g: f32, pose: &Pose) {
        if pose.trick != Some(Trick::FriendVisit) {
            return;
        }
        let u = pose.u;
        let slide = ease_out_cubic((u / 0.25).clamp(0.0, 1.0));
        let fade = if u > 0.85 { 1.0 - ((u - 0.85) / 0.15).clamp(0.0, 1.0) } else { 1.0 };
        let x = lerp(VW + 120.0, PET_X + 190.0, slide);
        let look = Look { fur: Color32::from_rgb(206, 206, 216), fur_dark: Color32::from_rgb(168, 168, 184), fur_light: Color32::from_rgb(228, 228, 236), belly: CREAM, scale: 0.62, seed: 3.7, stripes: true, level: 1 };
        let mut p2 = Pose { mood: Mood::Playful, blink: pose.blink, ear: 0.9, hop: (cx.time * 8.0).sin().abs() * 16.0 * pose.play, lean: -(pose.lean), pupil: 1.2, smile: 0.9, play: pose.play, trick: None, u, tail: cx.time * 7.0 };
        p2.blink = ((cx.time * 0.7) % 3.0 < 0.12) as u8 as f32;
        draw_pet(cx, x, g, &look, &p2, fade);
        if fade > 0.4 {
            cx.text_o(x, g - 190.0, Align2::CENTER_BOTTOM, "เพื่อนมาเล่น!", 20.0, Color32::from_rgb(255, 250, 235), rgba(120, 70, 90, 200.0 / 255.0), true);
        }
    }

    fn draw_trick_prop(&self, cx: &Cx, g: f32, pose: &Pose, look: &Look) {
        let Some(trick) = pose.trick else { return };
        let u = pose.u;
        let k = look.scale;
        let reach = ease_out_back((u / 0.2).clamp(0.0, 1.0));
        let fade = if u > 0.86 { 1.0 - ((u - 0.86) / 0.14).clamp(0.0, 1.0) } else { 1.0 };
        let al = fade;
        let head_y = g - 150.0 * k - pose.hop;
        match trick {
            Trick::HeadPet | Trick::PawFive => {
                // มือคนดูเอื้อมเข้ามาลูบหัว / แปะอุ้งเท้า
                let hx = lerp(VW + 90.0, PET_X + 26.0 * k, reach);
                let hy = if trick == Trick::HeadPet { head_y - 10.0 * k + (cx.time * 5.0).sin() * 7.0 * pose.play } else { g - 62.0 * k };
                let sleeve = Color32::from_rgb(126, 190, 235);
                // แขนยื่นมาจากขอบขวาจอเสมอ มือวางบนหัวแล้วนิ้วลู่ลงมาทางหน้าแมว
                cx.rect(hx + 30.0, hy - 18.0, VW + 80.0, hy + 20.0, 18.0, with_a(sleeve, al));
                cx.rect(hx + 26.0, hy - 21.0, hx + 46.0, hy + 23.0, 9.0, with_a(Color32::from_rgb(96, 160, 210), al));
                cx.ellipse(hx + 12.0, hy, 30.0, 22.0, with_a(Color32::from_rgb(255, 220, 196), al));
                for i in 0..4 {
                    let fx = hx - 16.0 + i as f32 * 11.0;
                    let fy = hy + 12.0 + (i as f32 * 0.9).sin() * 3.0;
                    cx.ellipse(fx, fy, 6.5, 12.0, with_a(Color32::from_rgb(255, 226, 204), al));
                }
                if trick == Trick::HeadPet && pose.play > 0.2 {
                    for i in 0..3 {
                        let t = (cx.time * 2.4 + i as f32 * 0.7) % 1.0;
                        cx.circle(hx - 20.0 - t * 26.0, hy - 20.0 - t * 30.0, 3.4 * (1.0 - t), with_a(HEART, al * (1.0 - t)));
                    }
                }
            }
            Trick::WandChase => {
                let sway = (cx.time * 3.2).sin();
                let tipx = PET_X + sway * 190.0 * pose.play;
                let tipy = head_y - 120.0 - (cx.time * 4.1).cos().abs() * 60.0 * pose.play;
                let handx = lerp(VW + 80.0, VW - 120.0, reach);
                cx.line((handx, g - 250.0), (tipx, tipy), 6.0, with_a(Color32::from_rgb(196, 140, 90), al));
                cx.line((tipx, tipy), (tipx + sway * 12.0, tipy + 34.0), 2.5, with_a(Color32::from_rgb(240, 230, 210), al));
                for i in 0..7 {
                    let a = (i as f32 / 7.0) * TAU + cx.time * 2.0;
                    let r = 22.0 + (i % 3) as f32 * 6.0;
                    cx.ellipse_rot(tipx + sway * 12.0 + a.cos() * r * 0.5, tipy + 40.0 + a.sin() * r * 0.4, 15.0, 5.5, a, with_a(if i % 2 == 0 { Color32::from_rgb(255, 150, 190) } else { Color32::from_rgb(255, 226, 150) }, al));
                }
                cx.circle(tipx + sway * 12.0, tipy + 40.0, 7.0, with_a(Color32::from_rgb(255, 250, 240), al));
            }
            Trick::DonutTreat => {
                let x = lerp(BOWL_X + 60.0, PET_X + 78.0 * k, reach);
                let y = g - 74.0 * k - (cx.time * 3.0).sin().abs() * 8.0 * pose.play;
                cx.circle(x, y, 30.0, with_a(Color32::from_rgb(226, 164, 106), al));
                cx.circle(x, y, 27.0, with_a(Color32::from_rgb(255, 178, 206), al));
                cx.circle(x, y, 10.0, with_a(Color32::from_rgb(250, 226, 200), al));
                for i in 0..8 {
                    let a = i as f32 / 8.0 * TAU;
                    cx.rect(x + a.cos() * 19.0 - 3.0, y + a.sin() * 19.0 - 1.5, x + a.cos() * 19.0 + 3.0, y + a.sin() * 19.0 + 1.5, 1.5, with_a(if i % 2 == 0 { Color32::from_rgb(255, 250, 240) } else { Color32::from_rgb(150, 220, 255) }, al));
                }
            }
            Trick::MilkBowl | Trick::MilkBottle => {
                let x = lerp(BOWL_X + 90.0, PET_X + 96.0 * k, reach);
                let y = g - 26.0;
                if trick == Trick::MilkBowl {
                    cx.poly(&[(x - 46.0, y - 34.0), (x + 46.0, y - 34.0), (x + 32.0, y), (x - 32.0, y)], with_a(Color32::from_rgb(255, 200, 214), al));
                    cx.ellipse(x, y - 34.0, 46.0, 12.0, with_a(Color32::from_rgb(255, 250, 244), al));
                    cx.ellipse(x, y - 34.0, 38.0, 9.0, with_a(Color32::from_rgb(255, 252, 246), al));
                } else {
                    let tilt = -0.5 * pose.play;
                    cx.ellipse_rot(x, y - 60.0, 24.0, 52.0, tilt, with_a(Color32::from_rgb(255, 252, 246), al));
                    cx.ellipse_rot(x, y - 60.0, 19.0, 44.0, tilt, with_a(Color32::from_rgb(255, 244, 224), al));
                    cx.rect(x - 12.0, y - 118.0, x + 12.0, y - 100.0, 5.0, with_a(Color32::from_rgb(255, 178, 196), al));
                    cx.ellipse(x, y - 122.0, 13.0, 10.0, with_a(Color32::from_rgb(255, 206, 176), al));
                }
                if pose.play > 0.3 {
                    for i in 0..4 {
                        let t = (cx.time * 2.0 + i as f32 * 0.4) % 1.0;
                        cx.circle(x - 30.0 + i as f32 * 8.0, y - 40.0 - t * 26.0, 3.0 * (1.0 - t), with_a(Color32::from_rgb(255, 255, 255), al * (1.0 - t)));
                    }
                }
            }
            Trick::FishToy => {
                let swim = (cx.time * 2.6).sin();
                let x = PET_X + swim * 150.0 * pose.play;
                let y = head_y - 60.0 - (cx.time * 3.4).cos().abs() * 40.0 * pose.play;
                let dir = if swim.cos() >= 0.0 { 1.0 } else { -1.0 };
                cx.ellipse(x, y, 34.0, 21.0, with_a(Color32::from_rgb(150, 214, 240), al));
                cx.poly(&[(x - 34.0 * dir, y), (x - 56.0 * dir, y - 16.0), (x - 56.0 * dir, y + 16.0)], with_a(Color32::from_rgb(120, 190, 226), al));
                cx.circle(x + 16.0 * dir, y - 5.0, 5.0, with_a(INK, al));
                cx.circle(x + 17.5 * dir, y - 6.5, 1.8, with_a(Color32::WHITE, al));
                for i in 0..3 {
                    cx.circle(x - 6.0 * dir + i as f32 * 9.0 * dir, y + 8.0, 3.0, with_a(Color32::from_rgb(255, 255, 255), al * 0.7));
                }
            }
            Trick::Knead => {
                // แมวนวดทำบิสกิต: อุ้งเท้าหน้าขยับบนผ้าห่ม
                let by = g - 18.0;
                cx.ellipse(PET_X, by + 6.0, 120.0 * k, 26.0, with_a(Color32::from_rgb(214, 178, 226), al));
                cx.ellipse(PET_X, by, 112.0 * k, 22.0, with_a(Color32::from_rgb(238, 214, 246), al));
                for i in 0..2 {
                    let px = PET_X + (i as f32 * 2.0 - 1.0) * 34.0 * k;
                    let press = ((cx.time * 3.4 + i as f32 * PI).sin() * 0.5 + 0.5) * pose.play;
                    cx.ellipse(px, by - 12.0 + press * 8.0, 20.0 * k, 13.0 * k, with_a(look.fur_light, al));
                    for t in 0..3 {
                        cx.circle(px - 8.0 * k + t as f32 * 8.0 * k, by - 18.0 + press * 8.0, 3.4 * k, with_a(PINK, al * 0.85));
                    }
                }
            }
            Trick::FriendVisit => {}
            Trick::GoldenKitten => {
                let pulse = 0.6 + 0.4 * (cx.time * 3.0).sin();
                cx.glow(PET_X, g - 110.0 * k, 250.0 * k, GOLD, 0.4 * al * pulse);
                for i in 0..14 {
                    let a = i as f32 / 14.0 * TAU + cx.time * 0.9;
                    let r = 150.0 * k + (cx.time * 2.0 + i as f32).sin() * 22.0;
                    let sx = PET_X + a.cos() * r;
                    let sy = g - 110.0 * k + a.sin() * r * 0.62;
                    draw_star(cx, sx, sy, 7.0 + (i % 3) as f32 * 3.0, with_a(GOLD, al * 0.9));
                }
                let ky = g - 250.0 * k - (cx.time * 1.6).sin() * 12.0;
                let kitten = Look { fur: Color32::from_rgb(255, 220, 130), fur_dark: Color32::from_rgb(226, 176, 70), fur_light: Color32::from_rgb(255, 240, 190), belly: CREAM, scale: 0.44, seed: 9.1, stripes: false, level: 1 };
                let kp = Pose { mood: Mood::Happy, blink: 0.8, ear: 1.0, hop: 0.0, lean: 0.0, pupil: 1.0, smile: 1.0, play: 0.0, trick: None, u: 0.0, tail: cx.time * 6.0 };
                draw_pet(cx, PET_X, ky + 90.0, &kitten, &kp, al);
                cx.text_o(PET_X, ky - 40.0, Align2::CENTER_BOTTOM, "ลูกแมวทองคำ!", 30.0, with_a(Color32::from_rgb(255, 246, 214), al), with_a(Color32::from_rgb(150, 96, 20), al), true);
            }
        }
    }

    fn draw_hearts(&self, cx: &Cx, g: f32) {
        for h in &self.hearts {
            let u = h.life / h.max;
            let a = (1.0 - u).clamp(0.0, 1.0);
            let y = g - 150.0 + h.y;
            draw_heart(cx, h.x, y, h.size * (0.7 + 0.3 * (u * PI).sin()), h.spin * h.life, with_a(HEART, a * 0.92));
        }
    }

    // ---------------------------------------------------------------- overlay

    pub fn draw_overlay(&self, cx: &Cx, card_title: &str, guide_label: &str, show_guide: bool) {
        self.draw_meters(cx);
        if let Some(a) = &self.active {
            self.draw_card(cx, a, card_title);
        }
        if self.level_flash > 0.0 {
            let a = (self.level_flash / 2.2).clamp(0.0, 1.0);
            let y = cx.vh * 0.45;
            cx.glow(PET_X, y, 300.0, GOLD, 0.35 * a);
            cx.text_o(PET_X, y, Align2::CENTER_CENTER, &format!("เลเวล {}!", self.level), 54.0, with_a(Color32::from_rgb(255, 248, 220), a), with_a(Color32::from_rgb(150, 96, 20), a), true);
            cx.text_o(PET_X, y + 48.0, Align2::CENTER_CENTER, level_title(self.level), 30.0, with_a(Color32::from_rgb(255, 240, 200), a), with_a(Color32::from_rgb(120, 70, 30), a), true);
        }
        if self.active.is_none() {
            self.draw_bubble(cx);
        }
        if show_guide {
            self.draw_guide(cx, guide_label);
        }
    }

    fn draw_meters(&self, cx: &Cx) {
        let x0 = 22.0;
        let y0 = 14.0;
        let w = 430.0;
        let row = 40.0;
        let h = row * 3.0 + 62.0;
        cx.rect(x0, y0, x0 + w, y0 + h, 16.0, rgba(38, 24, 40, 0.82));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 16.0, 2.5, rgba(255, 214, 232, 0.55));
        cx.text(x0 + 16.0, y0 + 26.0, Align2::LEFT_CENTER, &self.name, 23.0, Color32::from_rgb(255, 244, 228), true);
        let lv = format!("Lv.{} {}", self.level, level_title(self.level));
        cx.text(x0 + w - 16.0, y0 + 26.0, Align2::RIGHT_CENTER, &lv, 18.0, GOLD, true);
        let bars = [
            ("ความเหงา", self.loneliness, Color32::from_rgb(150, 190, 255), Color32::from_rgb(90, 130, 220), true),
            ("ความอิ่ม", self.fullness, Color32::from_rgb(255, 206, 130), Color32::from_rgb(232, 150, 70), false),
            ("ความผูกพัน", self.bond, Color32::from_rgb(255, 150, 190), Color32::from_rgb(226, 90, 140), false),
        ];
        for (i, (label, v, c1, c2, invert)) in bars.iter().enumerate() {
            let y = y0 + 52.0 + i as f32 * row;
            cx.text(x0 + 16.0, y + 13.0, Align2::LEFT_CENTER, label, 17.0, Color32::from_rgb(255, 240, 244), false);
            let bx0 = x0 + 116.0;
            let bx1 = x0 + w - 62.0;
            cx.rect(bx0, y + 4.0, bx1, y + 24.0, 10.0, rgba(255, 255, 255, 0.14));
            let u = (v / 100.0).clamp(0.0, 1.0);
            // เหงา: แถบยาว = แย่ จึงเตือนเป็นสีแดงเมื่อสูง
            let warn = if *invert { u > 0.7 } else { u < 0.25 };
            let fill = if warn { Color32::from_rgb(255, 110, 110) } else { *c1 };
            if u > 0.01 {
                cx.rect(bx0, y + 4.0, bx0 + (bx1 - bx0) * u, y + 24.0, 10.0, fill);
                cx.rect(bx0, y + 4.0, bx0 + (bx1 - bx0) * u, y + 12.0, 8.0, with_a(*c2, 0.35));
            }
            if warn {
                let pulse = 0.45 + 0.55 * (cx.time * 4.0).sin().abs();
                cx.rect_line(bx0, y + 4.0, bx1, y + 24.0, 10.0, 2.0, with_a(Color32::from_rgb(255, 140, 140), pulse));
            }
            cx.text(x0 + w - 16.0, y + 14.0, Align2::RIGHT_CENTER, &format!("{}", v.round() as u32), 17.0, Color32::WHITE, true);
        }
    }

    fn draw_bubble(&self, cx: &Cx) {
        let mood = self.mood();
        let text = mood.bubble();
        let k = look_for(self.level, self.seed).scale;
        let g = floor_y(cx.vh);
        let x = PET_X + 120.0 * k;
        let y = g - 210.0 * k;
        let size = cx.measure(text, 21.0, true);
        let w = size.x + 34.0;
        let h = 44.0;
        let bob = (cx.time * 1.7).sin() * 4.0;
        let col = match mood {
            Mood::Lonely => Color32::from_rgb(214, 228, 255),
            Mood::Hungry => Color32::from_rgb(255, 232, 200),
            _ => Color32::from_rgb(255, 240, 246),
        };
        cx.rect(x - w * 0.5, y - h + bob, x + w * 0.5, y + bob, 18.0, with_a(col, 0.95));
        cx.poly(&[(x - 26.0, y - 4.0 + bob), (x - 6.0, y - 4.0 + bob), (x - 22.0, y + 18.0 + bob)], with_a(col, 0.95));
        cx.text(x, y - h * 0.5 + bob, Align2::CENTER_CENTER, text, 21.0, INK, true);
    }

    fn draw_card(&self, cx: &Cx, a: &Interaction, title: &str) {
        let u = a.progress();
        let show = if u < 0.16 { u / 0.16 } else { 1.0 };
        let al = show.min(if u > 0.94 { (1.0 - u) / 0.06 } else { 1.0 }).clamp(0.0, 1.0);
        if al <= 0.02 {
            return;
        }
        let w = 620.0;
        let h = 100.0;
        let x0 = (VW - w) * 0.5;
        let y0 = 200.0;
        cx.rect(x0, y0, x0 + w, y0 + h, 20.0, with_a(rgba(44, 26, 44, 0.9), al));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 20.0, 3.0, with_a(if a.trick.is_star() { GOLD } else { PINK }, al * 0.9));
        cx.text(x0 + w * 0.5, y0 + 20.0, Align2::CENTER_CENTER, title, 19.0, with_a(Color32::from_rgb(255, 226, 240), al), true);
        cx.text(x0 + w * 0.5, y0 + 50.0, Align2::CENTER_CENTER, a.trick.label(), 30.0, with_a(Color32::from_rgb(255, 248, 232), al), true);
        let who = if a.combo > 1 { format!("{}  ×{}", a.user, a.combo) } else { a.user.clone() };
        cx.text(x0 + w * 0.5, y0 + 82.0, Align2::CENTER_CENTER, &who, 21.0, with_a(GOLD, al), true);
        if !a.gift.trim().is_empty() {
            let g = format!("{} · {} เพชร", a.gift, a.diamonds);
            cx.text(x0 + w - 16.0, y0 + 20.0, Align2::RIGHT_CENTER, &g, 15.0, with_a(Color32::from_rgb(226, 214, 232), al * 0.9), false);
        }
    }

    fn draw_guide(&self, cx: &Cx, label: &str) {
        let cols = 2;
        let rows = 5;
        let cw = 268.0;
        let rh = 27.0;
        let w = cw * cols as f32 + 26.0;
        let h = rh * rows as f32 + 46.0;
        let x0 = VW - w - 20.0;
        let y0 = 14.0;
        cx.rect(x0, y0, x0 + w, y0 + h, 14.0, rgba(38, 24, 40, 0.74));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 14.0, 2.0, rgba(255, 214, 232, 0.4));
        cx.text(x0 + w * 0.5, y0 + 22.0, Align2::CENTER_CENTER, label, 18.0, Color32::from_rgb(255, 232, 244), true);
        for (i, t) in TRICKS.iter().enumerate() {
            let c = i / rows;
            let r = i % rows;
            let x = x0 + 14.0 + c as f32 * cw;
            let y = y0 + 44.0 + r as f32 * rh;
            let active = self.active.as_ref().map(|a| a.trick == *t).unwrap_or(false);
            if active {
                cx.rect(x - 6.0, y - 11.0, x + cw - 12.0, y + 13.0, 8.0, rgba(255, 170, 200, 0.28));
            }
            cx.circle(x + 7.0, y, 6.0, if t.is_food() { Color32::from_rgb(255, 206, 130) } else { PINK });
            let txt = format!("{} · {}", t.label(), t.gift());
            cx.text(x + 20.0, y, Align2::LEFT_CENTER, &ellipsize(&txt, 24), 15.0, Color32::from_rgb(255, 244, 248), active);
            cx.text(x + cw - 20.0, y, Align2::RIGHT_CENTER, &format!("{}", t.diamonds()), 15.0, GOLD, true);
        }
    }
}

impl Default for PetStage {
    fn default() -> Self {
        Self::new()
    }
}

fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let t: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{t}…")
}

// --------------------------------------------------------------------- drawing

fn draw_paw(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    cx.ellipse(x, y + r * 0.32, r * 0.78, r * 0.62, c);
    for i in 0..4 {
        let a = -PI * 0.78 + i as f32 * (PI * 0.52 / 3.0);
        cx.ellipse(x + a.cos() * r * 0.72, y + a.sin() * r * 0.62, r * 0.26, r * 0.32, c);
    }
}

fn draw_heart(cx: &Cx, x: f32, y: f32, r: f32, ang: f32, c: Color32) {
    let (s, co) = ang.sin_cos();
    let p = |dx: f32, dy: f32| (x + dx * co - dy * s, y + dx * s + dy * co);
    let (a1, a2) = (p(-r * 0.42, -r * 0.18), p(r * 0.42, -r * 0.18));
    cx.circle(a1.0, a1.1, r * 0.5, c);
    cx.circle(a2.0, a2.1, r * 0.5, c);
    let (b, l, rr) = (p(0.0, r * 0.86), p(-r * 0.88, -r * 0.02), p(r * 0.88, -r * 0.02));
    cx.poly(&[l, rr, b], c);
}

fn draw_star(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    let mut pts = vec![];
    for i in 0..10 {
        let a = -PI / 2.0 + i as f32 * PI / 5.0;
        let rr = if i % 2 == 0 { r } else { r * 0.44 };
        pts.push((x + a.cos() * rr, y + a.sin() * rr));
    }
    cx.poly(&pts, c);
}

/// แมวชิบิ: หัวโต ตาโต กลมป้อม — สีหน้า/หู/หางเปลี่ยนตามอารมณ์และเลเวล
fn draw_pet(cx: &Cx, x: f32, g: f32, look: &Look, pose: &Pose, alpha: f32) {
    if alpha <= 0.02 {
        return;
    }
    let k = look.scale;
    let fur = with_a(look.fur, alpha);
    let dark = with_a(look.fur_dark, alpha);
    let light = with_a(look.fur_light, alpha);
    let belly = with_a(look.belly, alpha);
    let bob = (cx.time * 1.9 + look.seed * 4.0).sin() * 2.2 * k;
    let body_y = g - 52.0 * k - pose.hop - bob * 0.5;
    let head_y = body_y - 62.0 * k;
    let lean = pose.lean;

    cx.ellipse(x, g - 3.0, 62.0 * k, 12.0 * k, rgba(0, 0, 0, (54.0 * alpha) / 255.0));

    // tail: chubby segmented curve, wag speed follows mood
    let wag = pose.tail.sin() * if matches!(pose.mood, Mood::Playful | Mood::Happy) { 16.0 } else { 7.0 } * k;
    let curl = if pose.mood == Mood::Lonely { 1.0 } else { 0.0 };
    let t0 = (x - 44.0 * k, body_y + 16.0 * k);
    let n = 10;
    for i in 0..n {
        let u = i as f32 / (n - 1) as f32;
        let dx = -(6.0 + 46.0 * u) * k + wag * u * u;
        let dy = -6.0 * k - (1.0 - curl) * 44.0 * k * u + curl * 16.0 * k * u + (u * PI).sin() * 12.0 * k;
        let r = lerp(13.0, 5.0, u) * k;
        cx.circle(t0.0 + dx, t0.1 + dy, r + 1.6 * k, dark);
        cx.circle(t0.0 + dx, t0.1 + dy, r, fur);
        if i + 2 >= n {
            cx.circle(t0.0 + dx - 1.4 * k, t0.1 + dy - 1.4 * k, r * 0.5, light);
        }
    }

    // back legs / haunch
    cx.ellipse(x - 34.0 * k, body_y + 26.0 * k, 30.0 * k, 24.0 * k, dark);
    cx.ellipse(x + 34.0 * k, body_y + 26.0 * k, 30.0 * k, 24.0 * k, dark);

    // body: round chibi loaf
    cx.ellipse(x, body_y, 62.0 * k, 54.0 * k, dark);
    cx.ellipse(x, body_y - 1.5 * k, 59.0 * k, 51.0 * k, fur);
    cx.ellipse(x, body_y + 16.0 * k, 40.0 * k, 32.0 * k, belly);
    if look.stripes {
        for i in 0..3 {
            let sy = body_y - 26.0 * k + i as f32 * 17.0 * k;
            cx.ellipse_rot(x - 34.0 * k, sy, 13.0 * k, 4.5 * k, -0.35, with_a(look.fur_dark, alpha * 0.55));
            cx.ellipse_rot(x + 34.0 * k, sy, 13.0 * k, 4.5 * k, 0.35, with_a(look.fur_dark, alpha * 0.55));
        }
    }

    // front paws (toe beans)
    for s in [-1.0, 1.0] {
        let px = x + s * 24.0 * k;
        let py = body_y + 44.0 * k;
        cx.ellipse(px, py, 19.0 * k, 13.0 * k, light);
        for t in 0..3 {
            cx.circle(px - 7.0 * k + t as f32 * 7.0 * k, py - 3.0 * k, 2.6 * k, with_a(PINK, alpha * 0.75));
        }
    }

    // head
    let hx = x + lean * 40.0 * k;
    cx.circle(hx, head_y, 55.0 * k, dark);
    cx.circle(hx, head_y - 1.5 * k, 52.5 * k, fur);
    // ears: perk vs droop
    for s in [-1.0, 1.0] {
        let ex = hx + s * 36.0 * k;
        let ey = head_y - 38.0 * k;
        let droop = (1.0 - pose.ear) * 0.5;
        let twitch = if pose.mood == Mood::Playful { (cx.time * 6.0 + s).sin() * 3.0 * k } else { 0.0 };
        let tipx = ex + s * (10.0 - droop * 26.0) * k;
        let tipy = ey - (30.0 - droop * 34.0) * k + twitch;
        cx.poly(&[(ex - s * 16.0 * k, ey + 12.0 * k), (ex + s * 18.0 * k, ey + 8.0 * k), (tipx, tipy)], dark);
        cx.poly(&[(ex - s * 11.0 * k, ey + 8.0 * k), (ex + s * 12.0 * k, ey + 5.0 * k), (tipx - s * 2.0 * k, tipy + 7.0 * k)], with_a(PINK, alpha * 0.8));
    }
    // forehead stripes
    if look.stripes {
        for i in 0..3 {
            let sx = hx + (i as f32 - 1.0) * 12.0 * k;
            cx.ellipse_rot(sx, head_y - 34.0 * k, 3.6 * k, 11.0 * k, (i as f32 - 1.0) * 0.22, with_a(look.fur_dark, alpha * 0.6));
        }
    }
    // cheeks
    cx.circle(hx - 34.0 * k, head_y + 14.0 * k, 12.0 * k, with_a(PINK, alpha * 0.32));
    cx.circle(hx + 34.0 * k, head_y + 14.0 * k, 12.0 * k, with_a(PINK, alpha * 0.32));
    // muzzle
    cx.ellipse(hx, head_y + 20.0 * k, 24.0 * k, 17.0 * k, belly);

    // eyes
    let eye_y = head_y + 2.0 * k;
    for s in [-1.0, 1.0] {
        let ex = hx + s * 21.0 * k;
        let closed = pose.blink;
        if closed > 0.6 || pose.mood == Mood::Happy {
            // smiling closed eyes ^ ^
            cx.polyline(&[(ex - 11.0 * k, eye_y + 3.0 * k), (ex, eye_y - 6.0 * k), (ex + 11.0 * k, eye_y + 3.0 * k)], 4.0 * k, with_a(INK, alpha));
        } else if pose.mood == Mood::Lonely {
            cx.ellipse(ex, eye_y + 2.0 * k, 12.0 * k, 13.0 * k * (1.0 - closed * 0.85), with_a(INK, alpha));
            cx.circle(ex + 3.0 * k, eye_y - 2.0 * k, 3.6 * k, with_a(Color32::WHITE, alpha * 0.9));
            // droopy lid + tear
            cx.polyline(&[(ex - 13.0 * k, eye_y - 8.0 * k), (ex + 12.0 * k, eye_y - 3.0 * k)], 3.4 * k, with_a(look.fur_dark, alpha));
            let td = (cx.time * 0.9 + look.seed) % 1.6;
            if td < 1.0 {
                cx.circle(ex + s * 12.0 * k, eye_y + 12.0 * k + td * 26.0 * k, 3.6 * k * (1.0 - td * 0.5), with_a(Color32::from_rgb(150, 210, 255), alpha * (1.0 - td)));
            }
        } else {
            let ry = 16.0 * k * (1.0 - closed * 0.9);
            cx.ellipse(ex, eye_y, 13.0 * k, ry.max(1.2 * k), with_a(INK, alpha));
            let pr = 8.0 * k * pose.pupil.min(1.3);
            cx.circle(ex, eye_y + 1.0 * k, pr.min(ry * 0.8), with_a(Color32::from_rgb(70, 50, 60), alpha));
            cx.circle(ex - 4.0 * k, eye_y - 5.0 * k, 4.4 * k, with_a(Color32::WHITE, alpha * 0.95));
            cx.circle(ex + 4.5 * k, eye_y + 5.0 * k, 2.2 * k, with_a(Color32::WHITE, alpha * 0.6));
        }
    }
    // nose + mouth (ω)
    cx.poly(&[(hx - 5.0 * k, head_y + 15.0 * k), (hx + 5.0 * k, head_y + 15.0 * k), (hx, head_y + 21.0 * k)], with_a(PINK, alpha));
    let mw = 9.0 * k;
    let my = head_y + 24.0 * k;
    let smile = pose.smile;
    cx.polyline(&[(hx - mw, my - smile * 2.0 * k), (hx - mw * 0.4, my + 3.0 * k + smile * 2.0 * k), (hx, my), (hx + mw * 0.4, my + 3.0 * k + smile * 2.0 * k), (hx + mw, my - smile * 2.0 * k)], 2.8 * k, with_a(INK, alpha * 0.85));
    // whiskers
    for s in [-1.0, 1.0] {
        for i in 0..3 {
            let wy = head_y + 16.0 * k + (i as f32 - 1.0) * 6.0 * k;
            let droop = if pose.mood == Mood::Lonely { 5.0 * k } else { -2.0 * k };
            cx.line((hx + s * 24.0 * k, wy), (hx + s * 56.0 * k, wy + (i as f32 - 1.0) * 5.0 * k + droop), 1.9 * k, with_a(Color32::from_rgb(255, 255, 255), alpha * 0.75));
        }
    }

    // level accessories
    if look.level >= 2 {
        let cy = head_y + 46.0 * k;
        cx.rect(hx - 34.0 * k, cy, hx + 34.0 * k, cy + 11.0 * k, 5.0, with_a(Color32::from_rgb(226, 86, 120), alpha));
        cx.circle(hx, cy + 15.0 * k, 7.0 * k, with_a(GOLD, alpha));
        cx.circle(hx, cy + 15.0 * k, 3.0 * k, with_a(Color32::from_rgb(190, 140, 40), alpha));
    }
    if look.level >= 3 {
        // bow
        let bx = hx + 40.0 * k;
        let by = head_y - 34.0 * k;
        cx.poly(&[(bx, by), (bx - 17.0 * k, by - 11.0 * k), (bx - 17.0 * k, by + 11.0 * k)], with_a(Color32::from_rgb(255, 120, 170), alpha));
        cx.poly(&[(bx, by), (bx + 17.0 * k, by - 11.0 * k), (bx + 17.0 * k, by + 11.0 * k)], with_a(Color32::from_rgb(255, 120, 170), alpha));
        cx.circle(bx, by, 5.0 * k, with_a(Color32::from_rgb(255, 200, 220), alpha));
    }
    if look.level >= 5 {
        // crown: band + three spikes, each convex so cx.poly tessellates correctly
        // ขอบเข้ม + ส้มเข้ม เพราะขนเลเวล 5 เป็นสีทองใกล้เคียง GOLD จนมงกุฎกลืนหาย
        let cy = head_y - 56.0 * k;
        let gold = with_a(Color32::from_rgb(255, 158, 44), alpha);
        let edge = with_a(Color32::from_rgb(146, 94, 20), alpha);
        for (dx, tip) in [(-15.0 * k, -7.0 * k), (0.0, -15.0 * k), (15.0 * k, -7.0 * k)] {
            cx.poly(&[(hx + dx - 10.0 * k, cy + 5.0 * k), (hx + dx + 10.0 * k, cy + 5.0 * k), (hx + dx, cy + tip - 2.5 * k)], edge);
            cx.poly(&[(hx + dx - 7.0 * k, cy + 4.0 * k), (hx + dx + 7.0 * k, cy + 4.0 * k), (hx + dx, cy + tip)], gold);
        }
        cx.rect(hx - 27.0 * k, cy + 2.0 * k, hx + 27.0 * k, cy + 15.0 * k, 3.0 * k, edge);
        cx.rect(hx - 25.0 * k, cy + 4.0 * k, hx + 25.0 * k, cy + 13.0 * k, 2.5 * k, gold);
        for (dx, c) in [(-14.0 * k, Color32::from_rgb(255, 110, 150)), (0.0, Color32::from_rgb(150, 220, 255)), (14.0 * k, Color32::from_rgb(255, 110, 150))] {
            cx.circle(hx + dx, cy + 8.5 * k, 3.4 * k, with_a(c, alpha));
        }
        cx.circle(hx, cy - 15.0 * k, 4.0 * k, with_a(Color32::from_rgb(255, 110, 150), alpha));
        cx.glow(hx, cy, 60.0 * k, GOLD, 0.3 * alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stage() -> PetStage {
        let mut s = PetStage::with_seed(7);
        s.set_tuning("ทดสอบ", 20.0, 1.6, 0.9, 5, 30, 4.0, 14.0);
        s
    }

    fn inn(trick: Trick, user: &str) -> PetIn {
        PetIn { user: user.into(), tier: 0, gift: trick.gift().into(), gift_id: 1, diamonds: trick.diamonds(), trick }
    }

    #[test]
    fn ten_tricks_have_distinct_keys_prices_and_effects() {
        let mut keys = std::collections::BTreeSet::new();
        let mut prices = vec![];
        for t in TRICKS {
            assert!(keys.insert(t.key()), "duplicate key {}", t.key());
            assert_eq!(Trick::from_key(t.key()), Some(t));
            assert!(!t.label().is_empty());
            assert!(t.seconds() > 0.0);
            prices.push(t.diamonds());
        }
        assert_eq!(TRICKS.len(), 10);
        assert!(prices.windows(2).all(|w| w[0] < w[1]), "gimmicks must ascend by price: {prices:?}");
        assert_eq!(prices.first(), Some(&1));
        assert!(prices.last() >= Some(&1000));
    }

    #[test]
    fn loneliness_rises_only_after_the_idle_threshold() {
        let mut s = stage();
        s.loneliness = 0.0;
        s.tick(19.0);
        assert_eq!(s.loneliness, 0.0, "must not grow before lonely_after");
        s.tick(2.0);
        assert!(s.loneliness > 0.0, "must grow after lonely_after");
        let before = s.loneliness;
        s.enqueue(inn(Trick::HeadPet, "ก"));
        assert!(s.loneliness < before, "petting must reduce loneliness");
        assert_eq!(s.since_touch(), 0.0);
    }

    #[test]
    fn fullness_decays_and_feeding_restores_and_clamps() {
        let mut s = stage();
        s.fullness = 50.0;
        s.tick(10.0);
        assert!(s.fullness < 50.0, "fullness must decay");
        s.fullness = 95.0;
        s.enqueue(inn(Trick::MilkBottle, "ก"));
        assert_eq!(s.fullness, 100.0, "must clamp at 100");
        s.fullness = 10.0;
        s.clear();
        s.enqueue(inn(Trick::DonutTreat, "ข"));
        assert!(s.fullness > 10.0);
    }

    #[test]
    fn wand_play_costs_a_little_fullness() {
        let mut s = stage();
        s.fullness = 60.0;
        s.enqueue(inn(Trick::WandChase, "ก"));
        assert!(s.fullness < 60.0, "playing makes the cat hungry");
        assert!(s.loneliness < 12.0);
    }

    #[test]
    fn bond_levels_up_and_caps_at_max_level() {
        let mut s = stage();
        s.set_tuning("ทดสอบ", 20.0, 1.6, 0.9, 3, 30, 0.0, 14.0);
        for i in 0..40 {
            s.clear();
            s.enqueue(inn(Trick::Knead, &format!("u{i}")));
        }
        assert_eq!(s.level, 3, "must stop at max_level");
        assert!(s.bond <= 100.0);
        assert!(s.level_flash() >= 0.0);
    }

    #[test]
    fn golden_kitten_maxes_every_meter() {
        let mut s = stage();
        s.loneliness = 90.0;
        s.fullness = 5.0;
        s.enqueue(inn(Trick::GoldenKitten, "_star"));
        assert_eq!(s.loneliness, 0.0);
        assert_eq!(s.fullness, 100.0);
        assert!(s.bond > 0.0);
    }

    #[test]
    fn mood_follows_the_meters() {
        let mut s = stage();
        s.loneliness = 90.0;
        s.fullness = 90.0;
        assert_eq!(s.mood(), Mood::Lonely);
        s.loneliness = 10.0;
        s.fullness = 10.0;
        assert_eq!(s.mood(), Mood::Hungry);
        s.fullness = 90.0;
        assert_eq!(s.mood(), Mood::Happy);
        s.loneliness = 50.0;
        assert_eq!(s.mood(), Mood::Content);
        s.enqueue(inn(Trick::HeadPet, "ก"));
        assert_eq!(s.mood(), Mood::Playful);
    }

    #[test]
    fn clear_keeps_the_cat_and_its_meters() {
        let mut s = stage();
        s.enqueue(inn(Trick::HeadPet, "ก"));
        s.enqueue(inn(Trick::MilkBowl, "ข"));
        s.loneliness = 42.0;
        s.fullness = 55.0;
        s.bond = 30.0;
        s.level = 2;
        s.clear();
        assert!(!s.busy());
        assert_eq!(s.loneliness, 42.0);
        assert_eq!(s.fullness, 55.0);
        assert_eq!(s.level, 2, "the cat persists across clear");
        s.reset_stats();
        assert_eq!(s.level, 1);
        assert_eq!(s.bond, 0.0);
    }

    #[test]
    fn same_sender_and_trick_merge_into_a_combo() {
        let mut s = stage();
        s.enqueue(inn(Trick::HeadPet, "ก"));
        s.enqueue(inn(Trick::HeadPet, "ก"));
        s.enqueue(inn(Trick::HeadPet, "ก"));
        assert_eq!(s.combo(), 3);
        assert_eq!(s.len(), 1, "merged, not one entry per gift");
    }

    #[test]
    fn flood_stays_bounded_and_clamped() {
        let mut s = stage();
        for i in 0..500 {
            s.enqueue(inn(TRICKS[i % TRICKS.len()], &format!("u{}", i % 40)));
            s.tick(0.05);
        }
        assert!(s.live_units() <= 260, "live_units={}", s.live_units());
        assert!(s.queued().len() <= 30);
        assert!(s.loneliness >= 0.0 && s.loneliness <= 100.0);
        assert!(s.fullness >= 0.0 && s.fullness <= 100.0);
        assert!(s.bond >= 0.0 && s.bond <= 100.0);
        assert!(s.level <= 5);
        assert!(s.overflow_extra() > 0, "overflow must be counted, not stored");
    }

    #[test]
    fn interaction_phases_and_finish() {
        let mut s = stage();
        s.enqueue(inn(Trick::WandChase, "ก"));
        assert_eq!(s.lead().unwrap().phase(), "reach");
        s.tick(2.0);
        assert_eq!(s.lead().unwrap().phase(), "play");
        s.tick(2.4);
        assert_eq!(s.lead().unwrap().phase(), "thanks");
        s.tick(2.0);
        assert!(s.lead().is_none());
        assert_eq!(s.finished, 1);
        assert_eq!(s.started, 1);
    }

    #[test]
    fn queue_plays_in_order() {
        let mut s = stage();
        s.enqueue(inn(Trick::HeadPet, "A"));
        s.enqueue(inn(Trick::MilkBowl, "B"));
        s.enqueue(inn(Trick::FishToy, "C"));
        assert_eq!(s.lead().unwrap().user, "A");
        assert_eq!(s.queued().len(), 2);
        s.tick(3.0);
        assert_eq!(s.lead().unwrap().user, "B");
        s.tick(4.0);
        assert_eq!(s.lead().unwrap().user, "C");
    }

    #[test]
    fn builtin_ladder_matches_the_real_catalog_prices() {
        let empty = BTreeMap::new();
        for t in TRICKS {
            assert_eq!(trick_for(t.gift(), 0, t.diamonds(), &empty), t, "{}", t.key());
        }
    }

    #[test]
    fn unmapped_gift_falls_back_to_the_dearest_affordable_trick() {
        let empty = BTreeMap::new();
        assert_eq!(trick_for("Unknown", 0, 1, &empty), Trick::HeadPet);
        assert_eq!(trick_for("Unknown", 0, 9, &empty), Trick::PawFive);
        assert_eq!(trick_for("Unknown", 0, 10, &empty), Trick::WandChase);
        assert_eq!(trick_for("Unknown", 0, 399, &empty), Trick::Knead);
        assert_eq!(trick_for("Unknown", 0, 99999, &empty), Trick::GoldenKitten);
        assert_eq!(trick_for("Unknown", 0, 0, &empty), Trick::HeadPet);
    }

    #[test]
    fn overrides_win_by_name_or_id_and_ignore_unknown_keys() {
        let mut map = BTreeMap::new();
        map.insert("rose".to_string(), "golden_kitten".to_string());
        map.insert("5879".to_string(), "milk_bowl".to_string());
        assert_eq!(trick_for("Rose", 5655, 1, &map), Trick::GoldenKitten);
        assert_eq!(trick_for("Doughnut", 5879, 30, &map), Trick::MilkBowl);
        assert_eq!(trick_for("Rose", 5655, 1, &BTreeMap::new()), Trick::HeadPet);
        let mut bad = BTreeMap::new();
        bad.insert("Rose".to_string(), "not_a_trick".to_string());
        assert_eq!(trick_for("Rose", 5655, 1, &bad), Trick::HeadPet);
    }
}