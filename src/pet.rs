//! โหมดเลี้ยงแมว: แมวถาวรตัวหนึ่ง ผู้ชมช่วยกันเลี้ยงผ่านของขวัญ
//! ลูบหัว แปะอุ้งเท้า ไล่ไม้ขนกล่อ ให้อาหาร — มีแถบความเหงา / ความอิ่ม / ความผูกพัน (+เลเวล)
//! สีหน้าและท่าทางของแมวเปลี่ยนตามแถบ ไม่มีเลขมงคล
use crate::scene::*;
use egui::{Align2, Color32};
use std::collections::{BTreeMap, VecDeque};
use std::f32::consts::{PI, TAU};

const INK: Color32 = Color32::from_rgb(48, 34, 40);
const PINK: Color32 = Color32::from_rgb(255, 150, 175);
const GOLD: Color32 = Color32::from_rgb(255, 208, 96);
const HEART: Color32 = Color32::from_rgb(255, 105, 150);
const OUT: Color32 = Color32::from_rgb(58, 44, 44);
const WOOD: Color32 = Color32::from_rgb(196, 150, 108);
const WOOD_D: Color32 = Color32::from_rgb(140, 100, 66);

pub const PET_X: f32 = 540.0;
pub const BOWL_X: f32 = 780.0;
pub const BED_X: f32 = 270.0;
const WALK_MIN: f32 = 170.0;
const WALK_MAX: f32 = 910.0;

pub fn floor_y(vh: f32) -> f32 {
    (vh - 190.0_f32.min(vh * 0.2)).max(vh * 0.6)
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
            Mood::Happy => "เหมียว~ วันนี้ฟินมาก",
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
    patch: Option<Color32>,
    patch2: Option<Color32>,
    scale: f32,
    level: usize,
}

fn look_for(level: usize) -> Look {
    let white = Color32::from_rgb(255, 252, 246);
    let orange = Color32::from_rgb(242, 168, 86);
    let (fur, patch, patch2) = match level {
        1 | 2 => (white, Some(orange), None),
        3 | 4 => (white, Some(orange), Some(Color32::from_rgb(100, 76, 64))),
        _ => (Color32::from_rgb(255, 224, 140), Some(Color32::from_rgb(238, 158, 62)), None),
    };
    Look { fur, patch, patch2, scale: 2.1 + (level.min(5) as f32) * 0.08, level }
}

/// สิ่งที่แมวทำเองตอนไม่มีใครส่งของขวัญ (เดินเล่นไปมาทั่วห้อง)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Sit,
    Walk,
    Loaf,
    Sleep,
    Groom,
    Stretch,
    Bat,
    Sniff,
}

impl Act {
    pub fn key(self) -> &'static str {
        match self {
            Act::Sit => "sit",
            Act::Walk => "walk",
            Act::Loaf => "loaf",
            Act::Sleep => "sleep",
            Act::Groom => "groom",
            Act::Stretch => "stretch",
            Act::Bat => "bat",
            Act::Sniff => "sniff",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Form {
    Sit,
    Stand,
    Loaf,
    Sleep,
    Groom,
    PawUp,
    Stretch,
    Eat,
    Bat,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Eyes {
    Dot,
    Wide,
    Happy,
    Closed,
    Sad,
}

struct Pose {
    form: Form,
    eyes: Eyes,
    face: f32,
    walk: f32,
    moving: f32,
    hop: f32,
    wag: f32,
    t: f32,
    amt: f32,
    knead: bool,
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
    /// ตำแหน่งแมวในห้อง: x บนจอ, dy = ระยะลึกจากเส้นผนังลงมาบนพื้นทาทามิ
    cat_x: f32,
    cat_dy: f32,
    face: f32,
    act: Act,
    act_t: f32,
    act_dur: f32,
    walk_to: (f32, f32),
    then: Act,
    walk_ph: f32,
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
            cat_x: PET_X,
            cat_dy: 38.0,
            face: 1.0,
            act: Act::Sit,
            act_t: 0.0,
            act_dur: 2.5,
            walk_to: (PET_X, 38.0),
            then: Act::Sit,
            walk_ph: 0.0,
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
    pub fn activity(&self) -> Act {
        self.act
    }
    pub fn cat_x(&self) -> f32 {
        self.cat_x
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
        self.cat_x = PET_X;
        self.cat_dy = 38.0;
        self.face = 1.0;
        self.act = Act::Sit;
        self.act_t = 0.0;
        self.act_dur = 2.5;
        self.walk_to = (PET_X, 38.0);
        self.then = Act::Sit;
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
            x: self.cat_x + (self.rng.f32() - 0.5) * 150.0,
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
        self.wander(dt);

        for h in &mut self.hearts {
            h.life += dt;
            h.x += h.vx * dt;
            h.y += h.vy * dt;
            h.vy += 12.0 * dt;
            h.vx *= (-0.7 * dt).exp();
        }
        self.hearts.retain(|h| h.life < h.max);
    }

    // ------------------------------------------------------------------ roaming

    /// แมวเดินเล่นเองทั่วห้อง: นั่ง/นอน/เลียขน/ยืดตัว/ตะปบของเล่น — อารมณ์กำหนดทิศ
    fn wander(&mut self, dt: f32) {
        let info = self.active.as_ref().map(|a| (a.trick, a.progress()));
        if let Some((trick, u)) = info {
            let play = (u * PI).sin().clamp(0.0, 1.0);
            match trick {
                Trick::WandChase | Trick::FishToy => {
                    let tx = toy_x(self.clock, play);
                    let dx = tx - self.cat_x;
                    if dx.abs() > 6.0 {
                        self.face = if dx > 0.0 { 1.0 } else { -1.0 };
                    }
                    let step = dx.clamp(-260.0 * dt, 260.0 * dt);
                    self.cat_x += step;
                    self.cat_dy += (52.0 - self.cat_dy) * (dt * 2.0).min(1.0);
                    self.walk_ph += step.abs() * 0.11;
                    self.act = Act::Bat;
                }
                Trick::FriendVisit => {
                    self.face = if self.cat_x < VW * 0.5 { 1.0 } else { -1.0 };
                }
                _ => {
                    if self.step_to(PET_X, 34.0, 340.0, dt) {
                        self.act = Act::Sit;
                    }
                }
            }
            return;
        }
        if self.act == Act::Walk {
            let speed = if self.loneliness >= 70.0 { 55.0 } else { 88.0 };
            let (tx, ty) = self.walk_to;
            let arrived = self.step_to(tx, ty, speed, dt);
            self.act_t += dt;
            if arrived || self.act_t > 14.0 {
                self.act = self.then;
                self.act_t = 0.0;
                self.act_dur = self.dur_for(self.then);
                if self.then == Act::Sniff {
                    self.face = if BOWL_X > self.cat_x { 1.0 } else { -1.0 };
                }
            }
        } else {
            self.act_t += dt;
            if self.act_t >= self.act_dur {
                self.choose_act();
            }
        }
    }

    fn step_to(&mut self, tx: f32, ty: f32, speed: f32, dt: f32) -> bool {
        let dx = tx - self.cat_x;
        let dy = ty - self.cat_dy;
        let d = (dx * dx + dy * dy).sqrt();
        let step = speed * dt;
        if d <= step.max(6.0) {
            self.cat_x = tx;
            self.cat_dy = ty;
            return true;
        }
        self.cat_x += dx / d * step;
        self.cat_dy += dy / d * step;
        self.walk_ph += step * 0.11;
        if dx.abs() > 4.0 {
            self.face = if dx > 0.0 { 1.0 } else { -1.0 };
        }
        false
    }

    fn set_walk(&mut self, x: f32, dy: f32, then: Act) {
        self.walk_to = (x.clamp(WALK_MIN, WALK_MAX), dy.clamp(10.0, 95.0));
        self.then = then;
        self.act = Act::Walk;
        self.act_t = 0.0;
        let dx = self.walk_to.0 - self.cat_x;
        if dx.abs() > 4.0 {
            self.face = if dx > 0.0 { 1.0 } else { -1.0 };
        }
    }

    fn choose_act(&mut self) {
        self.act_t = 0.0;
        let r = self.rng.f32();
        if self.fullness <= 25.0 {
            let bx = BOWL_X + if self.cat_x < BOWL_X { -105.0 } else { 105.0 };
            if (self.cat_x - bx).abs() < 40.0 {
                self.act = Act::Sniff;
                self.face = if BOWL_X > self.cat_x { 1.0 } else { -1.0 };
            } else {
                self.set_walk(bx, 55.0, Act::Sniff);
                return;
            }
        } else if self.loneliness >= 70.0 {
            if r < 0.55 {
                self.act = Act::Sit;
            } else {
                self.set_walk(lerp(self.cat_x, PET_X, 0.5), 45.0, Act::Loaf);
                return;
            }
        } else if self.since_touch > 55.0 && r < 0.35 {
            self.set_walk(BED_X + 10.0, 30.0, Act::Sleep);
            return;
        } else if r < 0.30 {
            let x = WALK_MIN + self.rng.f32() * (WALK_MAX - WALK_MIN);
            let dy = 15.0 + self.rng.f32() * 75.0;
            let then = if self.rng.f32() < 0.5 { Act::Sit } else { Act::Loaf };
            self.set_walk(x, dy, then);
            return;
        } else if r < 0.45 {
            self.act = Act::Sit;
        } else if r < 0.58 {
            self.act = Act::Loaf;
        } else if r < 0.72 {
            self.act = Act::Groom;
        } else if r < 0.82 {
            self.act = Act::Stretch;
        } else if r < 0.92 {
            self.act = Act::Bat;
        } else {
            self.act = Act::Sit;
        }
        self.act_dur = self.dur_for(self.act);
    }

    fn dur_for(&mut self, a: Act) -> f32 {
        let r = self.rng.f32();
        match a {
            Act::Sit => 2.5 + r * 3.0,
            Act::Loaf => 4.0 + r * 4.0,
            Act::Sleep => 9.0 + r * 7.0,
            Act::Groom => 3.0 + r * 2.0,
            Act::Stretch => 2.2,
            Act::Bat => 2.5 + r * 1.5,
            Act::Sniff => 2.5 + r * 1.5,
            Act::Walk => 3.0,
        }
    }

    fn pose_of(&self) -> Pose {
        let blink = ((self.clock * 0.55 + self.seed * 7.0) % 3.6) < 0.13;
        let t = self.clock;
        let mut p = Pose { form: Form::Sit, eyes: Eyes::Dot, face: self.face, walk: self.walk_ph, moving: 0.0, hop: 0.0, wag: 0.0, t, knead: false, amt: 0.0 };
        match self.act {
            Act::Sit => {}
            Act::Walk => {
                p.form = Form::Stand;
                p.moving = 1.0;
            }
            Act::Loaf => p.form = Form::Loaf,
            Act::Sleep => {
                p.form = Form::Sleep;
                p.eyes = Eyes::Closed;
            }
            Act::Groom => {
                p.form = Form::Groom;
                p.eyes = Eyes::Closed;
            }
            Act::Stretch => {
                p.form = Form::Stretch;
                p.amt = ((self.act_t / 2.2).clamp(0.0, 1.0) * PI).sin();
                p.eyes = Eyes::Happy;
            }
            Act::Bat => {
                p.form = Form::Bat;
                p.hop = (self.act_t * 5.2).sin().abs() * 10.0;
                p.eyes = Eyes::Wide;
            }
            Act::Sniff => p.form = Form::Eat,
        }
        if let Some(a) = &self.active {
            let u = a.progress();
            let play = (u * PI).sin().clamp(0.0, 1.0);
            match a.trick {
                Trick::HeadPet => {
                    p.form = Form::Sit;
                    p.eyes = Eyes::Happy;
                    p.face = 1.0;
                    p.wag = 1.0;
                }
                Trick::PawFive => {
                    p.form = Form::PawUp;
                    p.eyes = Eyes::Wide;
                    p.face = 1.0;
                }
                Trick::WandChase | Trick::FishToy => {
                    let bat = (t * 5.2).sin() > 0.15;
                    p.form = if bat { Form::Bat } else { Form::Stand };
                    p.eyes = Eyes::Wide;
                    p.hop = if bat { (t * 5.2).sin().abs() * 12.0 * play } else { 0.0 };
                    p.moving = if bat { 0.0 } else { 1.0 };
                    p.wag = 1.0;
                }
                Trick::DonutTreat | Trick::MilkBowl => {
                    p.form = Form::Eat;
                    p.eyes = Eyes::Happy;
                    p.wag = 1.0;
                }
                Trick::MilkBottle => {
                    p.form = Form::Bat;
                    p.eyes = Eyes::Happy;
                    p.hop = (t * 4.0).sin().abs() * 4.0 * play;
                }
                Trick::FriendVisit => {
                    p.form = Form::Stand;
                    p.eyes = Eyes::Happy;
                    p.hop = (t * 8.0).sin().abs() * 16.0 * play;
                    p.wag = 1.0;
                }
                Trick::Knead => {
                    p.form = Form::Loaf;
                    p.eyes = Eyes::Closed;
                    p.knead = true;
                    p.wag = 0.6;
                }
                Trick::GoldenKitten => {
                    p.form = Form::Sit;
                    p.eyes = Eyes::Wide;
                    p.wag = 1.0;
                }
            }
        } else {
            match self.mood() {
                Mood::Lonely => {
                    if !matches!(p.eyes, Eyes::Closed) {
                        p.eyes = Eyes::Sad;
                    }
                }
                Mood::Happy => p.wag = 0.7,
                _ => {}
            }
        }
        if blink && !matches!(p.eyes, Eyes::Closed | Eyes::Happy) {
            p.eyes = Eyes::Closed;
        }
        p
    }

    fn cat_screen(&self, g: f32) -> (f32, f32) {
        (self.cat_x, g + self.cat_dy)
    }

    // ------------------------------------------------------------------ world

    pub fn draw_world(&self, cx: &Cx, props: &Cx, bg: &str) {
        let g = floor_y(cx.vh);
        let pose = self.pose_of();
        let look = look_for(self.level);
        if bg == "scene" {
            self.draw_room(cx, g);
        }
        self.draw_props(props, g);
        let (x, gy) = self.cat_screen(g);
        let lift = if pose.form == Form::Sleep { 12.0 } else { 0.0 };
        if cx.halo {
            cx.halo(x, gy - 70.0 * look.scale, 150.0 * look.scale, 100.0 * look.scale);
        }
        if let Some(a) = &self.active {
            if a.trick == Trick::Knead {
                self.draw_blanket(cx, PET_X, g + 34.0, &look, a.progress());
            }
        }
        self.draw_friend(cx, g);
        draw_cat(cx, x, gy - lift, &look, &pose, 1.0);
        if self.active.is_none() && self.act == Act::Bat {
            let py = gy - 95.0 * look.scale + (cx.time * 2.0).sin() * 12.0;
            sakura_petal(cx, x + pose.face * 44.0 * look.scale, py, 6.0, cx.time * 1.5, 0.9);
        }
        self.draw_trick_prop(cx, g, &pose, &look);
        self.draw_hearts(cx, g);
    }

    fn draw_room(&self, cx: &Cx, g: f32) {
        let h = cx.vh;
        let wall_h = g - 60.0;
        let night = (cx.time * 0.05).sin() > 0.0;
        // ผนังครีมอุ่น + ตารางโชจิบาง ๆ + คานบน (ไม่มีเสาผ่ากลางจอ)
        cx.vgradient(-40.0, -40.0, VW + 40.0, g, Color32::from_rgb(255, 248, 234), Color32::from_rgb(250, 235, 212));
        let grid = rgba(214, 190, 160, 0.16);
        let mut gx = 108.0;
        while gx < VW {
            cx.line((gx, 60.0), (gx, g - 120.0), 1.5, grid);
            gx += 216.0;
        }
        cx.line((-40.0, 60.0 + wall_h * 0.5), (VW + 40.0, 60.0 + wall_h * 0.5), 1.5, grid);
        cx.rect(-40.0, 36.0, VW + 40.0, 58.0, 0.0, WOOD);
        cx.rect(-40.0, 54.0, VW + 40.0, 58.0, 0.0, with_a(WOOD_D, 0.5));
        // บัวล่างไม้ + พื้นทาทามิหนาขึ้น มีเส้นลู่เข้าจุดรวมสายตา
        cx.rect(-40.0, g - 120.0, VW + 40.0, g - 12.0, 0.0, Color32::from_rgb(228, 196, 156));
        let mut wx = 60.0;
        while wx < VW {
            cx.line((wx, g - 120.0), (wx, g - 12.0), 2.0, rgba(170, 130, 92, 0.35));
            wx += 135.0;
        }
        cx.rect(-40.0, g - 124.0, VW + 40.0, g - 118.0, 0.0, WOOD);
        cx.rect(-40.0, g - 12.0, VW + 40.0, g, 0.0, WOOD_D);
        cx.vgradient(-40.0, g, VW + 40.0, h + 40.0, Color32::from_rgb(206, 214, 152), Color32::from_rgb(176, 188, 124));
        let mut yy = g + 7.0;
        while yy < h + 40.0 {
            cx.line((-40.0, yy), (VW + 40.0, yy), 1.0, rgba(120, 134, 78, 0.18));
            yy += 7.0;
        }
        let heri = rgba(74, 86, 108, 0.85);
        let (vpx, vpy) = (540.0, g - 420.0);
        let t0 = (g - vpy) / (h + 40.0 - vpy);
        let mut bx = -108.0;
        while bx <= VW + 108.0 {
            let xg = vpx + (bx - vpx) * t0;
            cx.polyline(&[(xg, g), (bx, h + 40.0)], 6.0, heri);
            bx += 216.0;
        }
        let hy = g + (h - g) * 0.45;
        cx.rect(-40.0, hy - 3.0, VW + 40.0, hy + 3.0, 0.0, heri);
        cx.rect(-40.0, g, VW + 40.0, g + 5.0, 0.0, heri);

        // หน้าต่างกลมมารุมาโดะ: ฟูจิ + ท้องฟ้า + กิ่งซากุระ + มู่ลี่ไม้ไผ่
        let (wxc, wyc, wr) = (200.0, 60.0 + wall_h * 0.62, 95.0);
        cx.circle(wxc, wyc, wr + 8.0, WOOD_D);
        cx.circle(wxc, wyc, wr + 3.0, WOOD);
        let r = wr - 4.0;
        let (top, bot) = if night {
            (Color32::from_rgb(46, 52, 108), Color32::from_rgb(120, 96, 160))
        } else {
            (Color32::from_rgb(150, 198, 242), Color32::from_rgb(255, 226, 234))
        };
        let n = 24;
        for i in 0..n {
            let y0 = wyc - r + 2.0 * r * i as f32 / n as f32;
            let y1 = wyc - r + 2.0 * r * (i + 1) as f32 / n as f32;
            let ym = (y0 + y1) * 0.5 - wyc;
            let hw = (r * r - ym * ym).max(0.0).sqrt();
            cx.rect(wxc - hw, y0, wxc + hw, y1 + 0.5, 0.0, mix(top, bot, i as f32 / n as f32));
        }
        let circ = ellipse_pts(wxc, wyc, r, r, 0.0, 40);
        if night {
            let moon = clip_convex(&ellipse_pts(wxc + 34.0, wyc - 40.0, 19.0, 19.0, 0.0, 20), &circ);
            if moon.len() >= 3 {
                cx.poly(&moon, Color32::from_rgb(255, 248, 214));
            }
            for (i, (dx, dy)) in [(-50.0, -20.0), (-20.0, -62.0), (8.0, -70.0), (60.0, -14.0), (-62.0, 22.0), (52.0, 38.0), (-32.0, 32.0), (14.0, -46.0)].iter().enumerate() {
                let tw = 0.4 + 0.6 * (cx.time * 2.0 + i as f32).sin().abs();
                cx.circle(wxc + dx, wyc + dy, 1.8, rgba(255, 255, 235, 0.8 * tw));
            }
        } else {
            cx.glow(wxc + 38.0, wyc - 44.0, 44.0, Color32::from_rgb(255, 240, 190), 0.8);
            let sun = clip_convex(&ellipse_pts(wxc + 38.0, wyc - 44.0, 18.0, 18.0, 0.0, 20), &circ);
            if sun.len() >= 3 {
                cx.poly(&sun, Color32::from_rgb(255, 246, 210));
            }
            for (cx0, cy0, rx0) in [(-44.0, -18.0, 26.0), (-16.0, -24.0, 20.0), (46.0, 6.0, 24.0)] {
                let cl = clip_convex(&ellipse_pts(wxc + cx0, wyc + cy0, rx0, rx0 * 0.38, 0.0, 18), &circ);
                if cl.len() >= 3 {
                    cx.poly(&cl, rgba(255, 255, 255, 0.85));
                }
            }
        }
        let fuji_c = if night { Color32::from_rgb(72, 78, 126) } else { Color32::from_rgb(112, 132, 182) };
        let fuji = clip_convex(&[(wxc - 78.0, wyc + 60.0), (wxc - 10.0, wyc - 32.0), (wxc + 10.0, wyc - 32.0), (wxc + 78.0, wyc + 60.0)], &circ);
        if fuji.len() >= 3 {
            cx.poly(&fuji, fuji_c);
        }
        let snow = clip_convex(&[(wxc - 10.0, wyc - 32.0), (wxc + 10.0, wyc - 32.0), (wxc + 20.0, wyc - 12.0), (wxc, wyc - 20.0), (wxc - 20.0, wyc - 12.0)], &circ);
        if snow.len() >= 3 {
            cx.fan((wxc, wyc - 24.0), &snow, rgba(255, 255, 255, 0.95));
        }
        for (hx0, hy0, hrx, hc) in [(-52.0, 64.0, 60.0, Color32::from_rgb(96, 148, 100)), (56.0, 68.0, 66.0, Color32::from_rgb(82, 132, 90))] {
            let hill = clip_convex(&ellipse_pts(wxc + hx0, wyc + hy0, hrx, hrx * 0.42, 0.0, 20), &circ);
            if hill.len() >= 3 {
                cx.poly(&hill, hc);
            }
        }
        let branch = Color32::from_rgb(110, 78, 60);
        cx.polyline(&[(wxc - 88.0, wyc - 30.0), (wxc - 40.0, wyc - 56.0), (wxc + 16.0, wyc - 50.0), (wxc + 50.0, wyc - 64.0)], 4.0, branch);
        cx.polyline(&[(wxc - 40.0, wyc - 56.0), (wxc - 28.0, wyc - 34.0)], 3.0, branch);
        for (bx0, by0) in [(-72.0, -38.0), (-44.0, -58.0), (-6.0, -52.0), (24.0, -54.0), (48.0, -66.0), (-28.0, -34.0)] {
            sakura_blossom(cx, wxc + bx0, wyc + by0, 6.5, 1.0);
        }
        // มู่ลี่ไม้ไผ่ม้วนอยู่ด้านบน
        for i in 0..4 {
            let sy0 = wyc - r + i as f32 * 6.5;
            let ym = sy0 + 3.0 - wyc;
            let hw = (r * r - ym * ym).max(0.0).sqrt();
            cx.rect(wxc - hw, sy0, wxc + hw, sy0 + 5.5, 0.0, Color32::from_rgb(214, 184, 132));
            cx.line((wxc - hw, sy0 + 5.5), (wxc + hw, sy0 + 5.5), 1.0, rgba(150, 116, 72, 0.6));
        }
        for s in [-1.0, 1.0] {
            cx.line((wxc + s * 40.0, wyc - r), (wxc + s * 40.0, wyc - r + 26.0), 1.5, rgba(120, 90, 56, 0.7));
        }
        cx.ring(wxc, wyc, wr + 0.5, 5.0, WOOD_D);

        // ชั้นไม้ + มาเนกิเนโกะ + หนังสือ + บอนไซ
        let sy = 60.0 + wall_h * 0.52;
        cx.rect(430.0, sy, 640.0, sy + 12.0, 3.0, WOOD);
        cx.rect(430.0, sy + 12.0, 640.0, sy + 16.0, 2.0, WOOD_D);
        cx.poly(&[(448.0, sy + 16.0), (462.0, sy + 16.0), (455.0, sy + 34.0)], WOOD_D);
        cx.poly(&[(608.0, sy + 16.0), (622.0, sy + 16.0), (615.0, sy + 34.0)], WOOD_D);
        let (mx, my) = (470.0, sy);
        let mwhite = Color32::from_rgb(255, 252, 246);
        cx.ellipse(mx, my - 14.0, 16.0, 17.0, OUT);
        cx.ellipse(mx, my - 14.0, 14.0, 15.0, mwhite);
        cx.poly(&[(mx - 12.0, my - 36.0), (mx - 2.0, my - 42.0), (mx - 13.0, my - 48.0)], OUT);
        cx.poly(&[(mx + 12.0, my - 36.0), (mx + 2.0, my - 42.0), (mx + 13.0, my - 48.0)], OUT);
        cx.poly(&[(mx - 10.0, my - 37.0), (mx - 4.0, my - 41.0), (mx - 11.0, my - 45.0)], mwhite);
        cx.poly(&[(mx + 10.0, my - 37.0), (mx + 4.0, my - 41.0), (mx + 11.0, my - 45.0)], mwhite);
        cx.circle(mx, my - 33.0, 13.5, OUT);
        cx.circle(mx, my - 33.0, 11.5, mwhite);
        let wave = (cx.time * 2.2).sin() * 0.18;
        cx.ellipse_rot(mx + 14.0, my - 38.0, 5.5, 10.0, 0.25 + wave, OUT);
        cx.ellipse_rot(mx + 14.0, my - 38.0, 4.0, 8.5, 0.25 + wave, mwhite);
        cx.circle(mx + 16.0, my - 45.0, 2.2, PINK);
        for s in [-1.0, 1.0] {
            cx.polyline(&[(mx + s * 7.0, my - 32.0), (mx + s * 3.5, my - 35.0), (mx + s * 1.0, my - 32.0)], 1.6, OUT);
        }
        cx.circle(mx, my - 29.0, 1.4, PINK);
        cx.rect(mx - 9.0, my - 24.0, mx + 9.0, my - 20.0, 2.0, Color32::from_rgb(214, 70, 60));
        cx.circle(mx, my - 17.0, 2.6, GOLD);
        cx.ellipse(mx, my - 6.0, 7.0, 4.5, GOLD);
        cx.ellipse(mx, my - 6.0, 4.5, 2.6, Color32::from_rgb(226, 168, 62));
        for (i, c) in [Color32::from_rgb(150, 190, 230), Color32::from_rgb(250, 200, 140), Color32::from_rgb(230, 150, 170)].iter().enumerate() {
            let bxx = 540.0 + i as f32 * 20.0;
            cx.rect(bxx, sy - 36.0, bxx + 15.0, sy, 2.0, OUT);
            cx.rect(bxx + 1.5, sy - 34.5, bxx + 13.5, sy, 1.5, *c);
        }
        let bnx = 615.0;
        cx.rect(bnx - 13.0, sy - 11.0, bnx + 13.0, sy, 2.0, OUT);
        cx.rect(bnx - 11.0, sy - 9.0, bnx + 11.0, sy, 1.5, Color32::from_rgb(190, 110, 80));
        cx.polyline(&[(bnx, sy - 10.0), (bnx - 2.0, sy - 24.0), (bnx + 6.0, sy - 32.0)], 3.0, branch);
        cx.circle(bnx - 8.0, sy - 30.0, 9.0, Color32::from_rgb(110, 160, 110));
        cx.circle(bnx + 7.0, sy - 37.0, 10.0, Color32::from_rgb(96, 148, 100));
        cx.circle(bnx + 1.0, sy - 24.0, 7.0, Color32::from_rgb(120, 172, 118));

        // นาฬิกาแมวคิตแคท: ตาเหลือกกลอกไปมา + หางลูกตุ้มแกว่ง
        let (kx, ky) = (535.0, sy - 92.0);
        let swing = (cx.time * 1.4).sin() * 0.45;
        let pend = (kx + swing.sin() * 44.0, ky + 28.0 + swing.cos() * 44.0);
        cx.line((kx, ky + 24.0), pend, 4.0, OUT);
        cx.circle(pend.0, pend.1, 8.0, OUT);
        cx.circle(pend.0, pend.1, 6.0, GOLD);
        cx.poly(&[(kx - 32.0, ky - 12.0), (kx - 8.0, ky - 26.0), (kx - 34.0, ky - 42.0)], OUT);
        cx.poly(&[(kx + 32.0, ky - 12.0), (kx + 8.0, ky - 26.0), (kx + 34.0, ky - 42.0)], OUT);
        cx.poly(&[(kx - 28.0, ky - 16.0), (kx - 12.0, ky - 25.0), (kx - 29.0, ky - 36.0)], INK);
        cx.poly(&[(kx + 28.0, ky - 16.0), (kx + 12.0, ky - 25.0), (kx + 29.0, ky - 36.0)], INK);
        cx.circle(kx, ky, 35.0, OUT);
        cx.circle(kx, ky, 32.0, INK);
        cx.circle(kx, ky + 4.0, 23.0, Color32::from_rgb(255, 250, 240));
        let look_x = (cx.time * 0.7).sin() * 5.0;
        for s in [-1.0, 1.0] {
            cx.circle(kx + s * 10.0, ky - 2.0, 7.5, OUT);
            cx.circle(kx + s * 10.0, ky - 2.0, 6.0, Color32::WHITE);
            cx.circle(kx + s * 10.0 + look_x, ky - 1.0, 3.0, INK);
            cx.line((kx + s * 16.0, ky + 8.0), (kx + s * 26.0, ky + 6.0 + s * 2.0), 1.4, OUT);
        }
        cx.poly(&[(kx - 2.5, ky + 6.0), (kx + 2.5, ky + 6.0), (kx, ky + 9.0)], PINK);
        cx.polyline(&[(kx - 6.0, ky + 12.0), (kx, ky + 15.0), (kx + 6.0, ky + 12.0)], 1.8, OUT);
        cx.poly(&[(kx - 13.0, ky + 30.0), (kx, ky + 24.0), (kx, ky + 36.0)], Color32::from_rgb(214, 70, 60));
        cx.poly(&[(kx + 13.0, ky + 30.0), (kx, ky + 24.0), (kx, ky + 36.0)], Color32::from_rgb(214, 70, 60));
        cx.circle(kx, ky + 30.0, 3.4, Color32::from_rgb(240, 110, 100));

        // โคมกระดาษแขวนจากขอเกี่ยวใต้แผง UI (เชือกไม่พาดผ่านไกด์)
        let lx = 740.0 + (cx.time * 0.8).sin() * 3.0;
        let ly = 60.0 + wall_h * 0.40;
        cx.rect(726.0, 236.0, 754.0, 244.0, 3.0, WOOD_D);
        cx.poly(&[(740.0, 244.0), (752.0, 244.0), (740.0, 262.0)], WOOD_D);
        cx.line((740.0, 244.0), (lx, ly - 44.0), 2.5, Color32::from_rgb(90, 60, 40));
        cx.glow_add(lx, ly, 74.0, Color32::from_rgb(255, 190, 110), if night { 0.75 } else { 0.45 });
        cx.rect(lx - 16.0, ly - 44.0, lx + 16.0, ly - 36.0, 3.0, Color32::from_rgb(90, 60, 40));
        cx.ellipse(lx, ly, 32.0, 40.0, OUT);
        cx.ellipse(lx, ly, 29.0, 37.0, Color32::from_rgb(255, 214, 150));
        cx.ellipse(lx - 8.0, ly - 6.0, 11.0, 24.0, rgba(255, 244, 214, 0.75));
        for i in 0..4 {
            let ry = ly - 24.0 + i as f32 * 16.0;
            let dy = (ry - ly) / 37.0;
            let hw = 29.0 * (1.0 - dy * dy).max(0.0).sqrt();
            cx.line((lx - hw, ry), (lx + hw, ry), 1.6, rgba(190, 120, 60, 0.6));
        }
        cx.rect(lx - 14.0, ly + 36.0, lx + 14.0, ly + 44.0, 3.0, Color32::from_rgb(90, 60, 40));
        draw_paw(cx, lx, ly + 4.0, 10.0, Color32::from_rgb(190, 90, 60));
        cx.line((lx, ly + 44.0), (lx, ly + 58.0), 2.0, Color32::from_rgb(214, 70, 60));

        // คัเคจิคุ: วงเอ็นโซ + อุ้งเท้า + ตราประทับ
        let (sx, sy2, sw, sh) = (880.0, 60.0 + wall_h * 0.48, 120.0, 104.0);
        cx.circle(sx + sw * 0.5, sy2 - 24.0, 3.0, WOOD_D);
        cx.polyline(&[(sx + sw * 0.5, sy2 - 24.0), (sx + 10.0, sy2 - 6.0)], 2.0, WOOD_D);
        cx.polyline(&[(sx + sw * 0.5, sy2 - 24.0), (sx + sw - 10.0, sy2 - 6.0)], 2.0, WOOD_D);
        cx.rect(sx + 6.0, sy2, sx + sw - 6.0, sy2 + sh, 2.0, Color32::from_rgb(255, 250, 238));
        cx.rect_line(sx + 6.0, sy2, sx + sw - 6.0, sy2 + sh, 2.0, 1.5, rgba(150, 110, 72, 0.5));
        cx.rect(sx - 4.0, sy2 - 8.0, sx + sw + 4.0, sy2 + 2.0, 4.0, WOOD_D);
        cx.rect(sx - 8.0, sy2 + sh - 2.0, sx + sw + 8.0, sy2 + sh + 10.0, 5.0, WOOD_D);
        let enso = arc_pts(sx + sw * 0.5, sy2 + sh * 0.42, 26.0, 26.0, 0.4, TAU - 0.3, 26);
        cx.polyline(&enso, 6.0, rgba(70, 52, 48, 0.85));
        draw_paw(cx, sx + sw * 0.5, sy2 + sh * 0.44, 10.0, rgba(70, 52, 48, 0.45));
        cx.rect(sx + sw - 30.0, sy2 + sh - 26.0, sx + sw - 14.0, sy2 + sh - 10.0, 2.0, Color32::from_rgb(214, 70, 60));

        // ราวซากุระพาดผนังซ้าย + แทงะคุ (ใต้แผงสถานะ ไม่ทับ UI)
        let gy2 = 248.0;
        let mut pts = Vec::new();
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            pts.push((30.0 + t * 440.0, gy2 + (t * PI).sin() * 14.0));
        }
        cx.polyline(&pts, 3.5, branch);
        let tanzaku = [Color32::from_rgb(255, 170, 190), Color32::from_rgb(170, 210, 255), Color32::from_rgb(255, 224, 150)];
        for i in 0..7 {
            let t = (i as f32 + 0.5) / 7.0;
            let x = 30.0 + t * 440.0;
            let y = gy2 + (t * PI).sin() * 14.0;
            sakura_blossom(cx, x, y - 3.0, 7.0, 0.95);
            if i % 3 == 1 {
                let sway = (cx.time * 1.3 + i as f32).sin() * 3.0;
                cx.rect(x - 5.0 + sway, y + 5.0, x + 5.0 + sway, y + 26.0, 2.0, with_a(tanzaku[(i / 3) % 3], 0.9));
            }
        }

        // กลีบซากุระร่วงทั้งฉาก
        for i in 0..6 {
            let t = cx.time * 0.1 + i as f32 * 0.37;
            let px = ((t * 120.0 + i as f32 * 173.0) % (VW + 60.0)) - 30.0;
            let py = ((t * 70.0 + i as f32 * 97.0) % (h + 40.0)) - 20.0;
            let sway = (cx.time * 1.1 + i as f32 * 2.0).sin() * 14.0;
            sakura_petal(cx, px + sway, py, 5.0 + (i % 3) as f32, t * 2.4 + i as f32, 0.55);
        }
    }

    fn draw_props(&self, cx: &Cx, g: f32) {
        // ซาบุตงเบาะประจำของแมว
        let zy = g + 34.0;
        cx.ellipse(PET_X, zy + 14.0, 106.0, 12.0, rgba(0, 0, 0, 0.15));
        cx.rect(PET_X - 100.0, zy - 20.0, PET_X + 100.0, zy + 20.0, 12.0, OUT);
        cx.rect(PET_X - 97.0, zy - 17.0, PET_X + 97.0, zy + 17.0, 10.0, Color32::from_rgb(108, 120, 184));
        cx.rect(PET_X - 84.0, zy - 11.0, PET_X + 84.0, zy + 11.0, 7.0, Color32::from_rgb(132, 144, 206));
        cx.rect_line(PET_X - 97.0, zy - 17.0, PET_X + 97.0, zy + 17.0, 10.0, 2.0, rgba(255, 255, 255, 0.3));
        for s in [-1.0, 1.0] {
            cx.circle(PET_X + s * 90.0, zy, 3.0, rgba(255, 255, 255, 0.5));
        }
        // คอนโดแมวสองชั้น + พู่ Pompom
        let tx = 985.0;
        cx.ellipse(tx, g + 20.0, 56.0, 9.0, rgba(0, 0, 0, 0.15));
        cx.rect(tx - 45.0, g + 8.0, tx + 45.0, g + 22.0, 6.0, OUT);
        cx.rect(tx - 42.0, g + 10.0, tx + 42.0, g + 20.0, 5.0, WOOD);
        cx.rect(tx - 11.0, g - 140.0, tx + 11.0, g + 10.0, 4.0, OUT);
        cx.rect(tx - 8.0, g - 138.0, tx + 8.0, g + 8.0, 3.0, Color32::from_rgb(222, 196, 150));
        for i in 0..11 {
            let y = g - 132.0 + i as f32 * 12.0;
            cx.line((tx - 8.0, y), (tx + 8.0, y + 4.0), 1.6, rgba(170, 136, 90, 0.7));
        }
        cx.rect(tx - 55.0, g - 152.0, tx + 55.0, g - 138.0, 5.0, OUT);
        cx.rect(tx - 52.0, g - 150.0, tx + 52.0, g - 140.0, 4.0, WOOD);
        cx.ellipse(tx, g - 152.0, 46.0, 9.0, Color32::from_rgb(255, 178, 196));
        cx.rect(tx - 9.0, g - 232.0, tx + 9.0, g - 150.0, 4.0, OUT);
        cx.rect(tx - 6.0, g - 230.0, tx + 6.0, g - 152.0, 3.0, Color32::from_rgb(222, 196, 150));
        cx.rect(tx - 40.0, g - 244.0, tx + 40.0, g - 230.0, 5.0, OUT);
        cx.rect(tx - 37.0, g - 242.0, tx + 37.0, g - 232.0, 4.0, WOOD);
        cx.ellipse(tx, g - 244.0, 32.0, 7.0, Color32::from_rgb(170, 210, 255));
        let sw = (cx.time * 1.6).sin() * 8.0;
        cx.line((tx + 48.0, g - 140.0), (tx + 48.0 + sw, g - 104.0), 2.0, Color32::from_rgb(150, 110, 70));
        cx.circle(tx + 48.0 + sw, g - 98.0, 9.0, OUT);
        cx.circle(tx + 48.0 + sw, g - 98.0, 7.0, PINK);
        // เสากระทบพันเชือก
        let sx = 90.0;
        cx.rect(sx - 28.0, g + 2.0, sx + 28.0, g + 16.0, 6.0, OUT);
        cx.rect(sx - 25.0, g + 4.0, sx + 25.0, g + 14.0, 5.0, WOOD_D);
        cx.rect(sx - 12.0, g - 150.0, sx + 12.0, g + 6.0, 5.0, OUT);
        cx.rect(sx - 9.0, g - 148.0, sx + 9.0, g + 4.0, 4.0, Color32::from_rgb(222, 196, 150));
        for i in 0..12 {
            let y = g - 144.0 + i as f32 * 12.0;
            cx.line((sx - 9.0, y), (sx + 9.0, y + 4.0), 1.6, rgba(170, 136, 90, 0.7));
        }
        cx.circle(sx, g - 158.0, 18.0, OUT);
        cx.circle(sx, g - 158.0, 15.0, Color32::from_rgb(255, 178, 196));
        // ฟูกนอนแมวขอบหนา
        cx.ellipse(BED_X, g + 42.0, 104.0, 12.0, rgba(0, 0, 0, 0.15));
        cx.ellipse(BED_X, g + 34.0, 104.0, 32.0, OUT);
        cx.ellipse(BED_X, g + 34.0, 100.0, 29.0, Color32::from_rgb(214, 150, 170));
        cx.ellipse(BED_X, g + 30.0, 78.0, 20.0, Color32::from_rgb(255, 226, 236));
        cx.ellipse(BED_X, g + 32.0, 64.0, 14.0, Color32::from_rgb(255, 244, 246));
        draw_paw(cx, BED_X, g + 52.0, 8.0, rgba(255, 255, 255, 0.6));
        // กล่องกระดาษ + ไหมพรม
        let bxx = 888.0;
        cx.ellipse(bxx, g + 80.0, 52.0, 8.0, rgba(0, 0, 0, 0.15));
        cx.poly(&[(bxx - 42.0, g + 26.0), (bxx - 8.0, g + 26.0), (bxx - 20.0, g + 4.0), (bxx - 50.0, g + 10.0)], OUT);
        cx.poly(&[(bxx - 39.0, g + 24.0), (bxx - 11.0, g + 24.0), (bxx - 21.0, g + 7.0), (bxx - 46.0, g + 12.0)], Color32::from_rgb(214, 178, 130));
        cx.poly(&[(bxx + 42.0, g + 26.0), (bxx + 8.0, g + 26.0), (bxx + 20.0, g + 4.0), (bxx + 50.0, g + 10.0)], OUT);
        cx.poly(&[(bxx + 39.0, g + 24.0), (bxx + 11.0, g + 24.0), (bxx + 21.0, g + 7.0), (bxx + 46.0, g + 12.0)], Color32::from_rgb(214, 178, 130));
        cx.rect(bxx - 44.0, g + 22.0, bxx + 44.0, g + 80.0, 4.0, OUT);
        cx.rect(bxx - 41.0, g + 25.0, bxx + 41.0, g + 77.0, 3.0, Color32::from_rgb(226, 192, 146));
        cx.rect(bxx - 6.0, g + 25.0, bxx + 6.0, g + 77.0, 0.0, rgba(240, 220, 180, 0.9));
        draw_paw(cx, bxx - 22.0, g + 52.0, 9.0, rgba(226, 120, 80, 0.8));
        let yx = bxx + 56.0;
        cx.circle(yx, g + 70.0, 13.0, OUT);
        cx.circle(yx, g + 70.0, 11.0, Color32::from_rgb(255, 138, 168));
        cx.polyline(&[(yx - 9.0, g + 66.0), (yx, g + 62.0), (yx + 9.0, g + 67.0)], 1.6, rgba(220, 90, 120, 0.8));
        cx.polyline(&[(yx - 10.0, g + 73.0), (yx, g + 77.0), (yx + 10.0, g + 72.0)], 1.6, rgba(220, 90, 120, 0.8));
        // ชามอาหาร: ปริมาณตามความอิ่ม
        let f = self.fullness / 100.0;
        cx.ellipse(BOWL_X, g + 48.0, 56.0, 9.0, rgba(0, 0, 0, 0.15));
        cx.poly(&[(BOWL_X - 56.0, g + 14.0), (BOWL_X + 56.0, g + 14.0), (BOWL_X + 40.0, g + 48.0), (BOWL_X - 40.0, g + 48.0)], OUT);
        cx.poly(&[(BOWL_X - 52.0, g + 17.0), (BOWL_X + 52.0, g + 17.0), (BOWL_X + 37.0, g + 45.0), (BOWL_X - 37.0, g + 45.0)], Color32::from_rgb(255, 246, 234));
        cx.poly(&[(BOWL_X - 48.0, g + 26.0), (BOWL_X + 48.0, g + 26.0), (BOWL_X + 45.0, g + 33.0), (BOWL_X - 45.0, g + 33.0)], Color32::from_rgb(120, 170, 220));
        cx.ellipse(BOWL_X, g + 15.0, 52.0, 12.0, OUT);
        cx.ellipse(BOWL_X, g + 15.0, 49.0, 10.0, Color32::from_rgb(226, 120, 110));
        cx.ellipse(BOWL_X, g + 15.0, 42.0, 8.0, Color32::from_rgb(120, 80, 60));
        if f > 0.02 {
            let ff = f.max(0.25);
            cx.ellipse(BOWL_X, g + 15.0, 42.0 * ff, 8.0 * ff, Color32::from_rgb(255, 248, 232));
            for i in 0..5 {
                let kk = i as f32;
                cx.circle(BOWL_X - 24.0 + kk * 12.0, g + 13.0 + (kk * 1.7).sin() * 2.5, 4.5, Color32::from_rgb(214, 164, 110));
            }
        }
        draw_paw(cx, BOWL_X, g + 39.0, 7.0, rgba(226, 120, 110, 0.7));
    }

    fn draw_friend(&self, cx: &Cx, g: f32) {
        let Some(a) = self.active.as_ref().filter(|a| a.trick == Trick::FriendVisit) else { return };
        let u = a.progress();
        let slide = ease_out_cubic((u / 0.25).clamp(0.0, 1.0));
        let fade = if u > 0.85 { 1.0 - ((u - 0.85) / 0.15).clamp(0.0, 1.0) } else { 1.0 };
        let from_right = self.cat_x < VW * 0.5;
        let target = self.cat_x + if from_right { 190.0 } else { -190.0 };
        let start = if from_right { VW + 120.0 } else { -120.0 };
        let x = lerp(start, target, slide);
        let look = Look { fur: Color32::from_rgb(206, 206, 216), patch: Some(Color32::from_rgb(150, 150, 170)), patch2: None, scale: look_for(self.level).scale * 0.78, level: 1 };
        let p2 = Pose { form: Form::Stand, eyes: if ((cx.time * 0.7) % 3.0) < 0.12 { Eyes::Closed } else { Eyes::Happy }, face: if from_right { -1.0 } else { 1.0 }, walk: cx.time * 6.0, moving: 0.0, hop: (cx.time * 8.0).sin().abs() * 14.0 * (u * PI).sin(), wag: 1.0, t: cx.time, knead: false, amt: 0.0 };
        draw_cat(cx, x, g + 55.0, &look, &p2, fade);
        if fade > 0.4 {
            cx.text_o(x, g + 55.0 - 130.0 * look.scale, Align2::CENTER_BOTTOM, "เพื่อนมาเล่น!", 20.0, Color32::from_rgb(255, 250, 235), rgba(120, 70, 90, 200.0 / 255.0), true);
        }
    }

    fn draw_blanket(&self, cx: &Cx, x: f32, gy: f32, look: &Look, u: f32) {
        let al = prop_fade(u);
        let k = look.scale;
        cx.ellipse(x, gy + 2.0, 56.0 * k, 17.0 * k, with_a(OUT, al * 0.9));
        cx.ellipse(x, gy, 54.0 * k, 15.0 * k, with_a(Color32::from_rgb(214, 178, 226), al));
        cx.ellipse(x, gy - 2.0, 46.0 * k, 11.0 * k, with_a(Color32::from_rgb(238, 214, 246), al));
        for s in [-1.0, 1.0] {
            cx.polyline(&[(x + s * 20.0 * k, gy - 8.0 * k), (x + s * 30.0 * k, gy + 4.0 * k)], 2.0, with_a(Color32::from_rgb(180, 140, 200), al * 0.7));
        }
    }

    fn draw_trick_prop(&self, cx: &Cx, g: f32, pose: &Pose, look: &Look) {
        let Some(a) = &self.active else { return };
        let u = a.progress();
        let play = (u * PI).sin().clamp(0.0, 1.0);
        let al = prop_fade(u);
        let reach = ease_out_back((u / 0.2).clamp(0.0, 1.0));
        let k = look.scale;
        let (x, gy) = self.cat_screen(g);
        let (hx, hy) = head_screen(x, gy, look, pose);
        let t = cx.time;
        match a.trick {
            Trick::HeadPet => {
                let handx = lerp(VW + 90.0, hx + 6.0 * k, reach);
                let handy = hy - 22.0 * k + (t * 5.0).sin() * 4.0 * play;
                draw_hand(cx, handx, handy, al, (t * 5.0).sin().abs() * play);
                if play > 0.3 {
                    for i in 0..3 {
                        let tt = (t * 2.4 + i as f32 * 0.7) % 1.0;
                        cx.circle(handx - 24.0 - tt * 26.0, handy - 16.0 - tt * 30.0, 3.4 * (1.0 - tt), with_a(HEART, al * (1.0 - tt)));
                    }
                }
            }
            Trick::PawFive => {
                let pawx = x + 33.0 * k;
                let pawy = gy - pose.hop - 88.0 * k;
                let handx = lerp(VW + 90.0, pawx + 16.0 * k, reach);
                let handy = pawy - 8.0 * k;
                draw_hand(cx, handx, handy, al, 0.0);
                if play > 0.5 {
                    let tw = (t * 4.0).sin().abs();
                    sparkle(cx, pawx + 8.0 * k, pawy - 14.0 * k, 10.0 * tw, al * tw);
                }
            }
            Trick::WandChase => {
                let tx = toy_x(t, play);
                let ty = gy - 120.0 * k - (t * 4.1).cos().abs() * 50.0 * play;
                let sway = (t * 3.2).sin();
                let handx = lerp(VW + 80.0, VW - 120.0, reach);
                let handy = gy - 260.0;
                cx.line((handx, handy), (tx, ty), 5.0, with_a(WOOD, al));
                cx.line((tx, ty), (tx + sway * 8.0, ty + 30.0), 2.0, with_a(Color32::from_rgb(240, 230, 210), al));
                let fx = tx + sway * 8.0;
                let fy = ty + 38.0;
                for i in 0..5 {
                    let ang = -PI / 2.0 + (i as f32 - 2.0) * 0.5 + sway * 0.2;
                    let c = if i % 2 == 0 { Color32::from_rgb(255, 150, 190) } else { Color32::from_rgb(255, 226, 150) };
                    cx.ellipse_rot(fx + ang.cos() * 10.0, fy + ang.sin() * 10.0 + 8.0, 14.0, 5.5, ang + PI / 2.0, with_a(OUT, al));
                    cx.ellipse_rot(fx + ang.cos() * 10.0, fy + ang.sin() * 10.0 + 8.0, 12.0, 4.0, ang + PI / 2.0, with_a(c, al));
                }
                cx.circle(fx, fy, 6.5, with_a(OUT, al));
                cx.circle(fx, fy, 5.0, with_a(GOLD, al));
                draw_hand(cx, handx, handy, al, 0.0);
            }
            Trick::FishToy => {
                let swim = (t * 2.6).sin();
                let tx = toy_x(t, play);
                let ty = gy - 60.0 * k - (t * 3.4).cos().abs() * 40.0 * play;
                let dir = if swim.cos() >= 0.0 { 1.0 } else { -1.0 };
                cx.poly(&[(tx - 28.0 * dir, ty), (tx - 50.0 * dir, ty - 15.0), (tx - 50.0 * dir, ty + 15.0)], with_a(OUT, al));
                cx.poly(&[(tx - 28.0 * dir, ty), (tx - 46.0 * dir, ty - 11.0), (tx - 46.0 * dir, ty + 11.0)], with_a(Color32::from_rgb(120, 190, 226), al));
                cx.ellipse(tx, ty, 32.0, 20.0, with_a(OUT, al));
                cx.ellipse(tx, ty, 29.0, 17.0, with_a(Color32::from_rgb(150, 214, 240), al));
                cx.circle(tx + 15.0 * dir, ty - 5.0, 4.5, with_a(OUT, al));
                cx.circle(tx + 16.5 * dir, ty - 6.5, 1.6, with_a(Color32::WHITE, al));
                for i in 0..3 {
                    cx.circle(tx - 6.0 * dir + i as f32 * 8.0 * dir, ty + 7.0, 2.6, with_a(Color32::WHITE, al * 0.7));
                }
            }
            Trick::DonutTreat => {
                let dx = x + pose.face * 52.0 * k;
                let dy = gy - 26.0 * k;
                let r = 24.0 * (1.0 - u * 0.45);
                cx.circle(dx, dy, r + 2.5, with_a(OUT, al));
                cx.circle(dx, dy, r, with_a(Color32::from_rgb(226, 164, 106), al));
                cx.circle(dx, dy, r * 0.86, with_a(Color32::from_rgb(255, 178, 206), al));
                cx.circle(dx, dy, r * 0.3, with_a(Color32::from_rgb(180, 120, 70), al));
                for i in 0..8 {
                    let ang = i as f32 / 8.0 * TAU;
                    let c = if i % 2 == 0 { Color32::from_rgb(255, 250, 240) } else { Color32::from_rgb(150, 220, 255) };
                    cx.rect(dx + ang.cos() * r * 0.6 - 2.5, dy + ang.sin() * r * 0.6 - 1.2, dx + ang.cos() * r * 0.6 + 2.5, dy + ang.sin() * r * 0.6 + 1.2, 1.2, with_a(c, al));
                }
            }
            Trick::MilkBowl => {
                let bx = x + pose.face * 58.0 * k;
                let by = gy;
                cx.poly(&[(bx - 40.0, by - 28.0), (bx + 40.0, by - 28.0), (bx + 28.0, by), (bx - 28.0, by)], with_a(OUT, al));
                cx.poly(&[(bx - 36.0, by - 25.0), (bx + 36.0, by - 25.0), (bx + 25.0, by - 2.0), (bx - 25.0, by - 2.0)], with_a(Color32::from_rgb(255, 200, 214), al));
                cx.ellipse(bx, by - 28.0, 40.0, 10.0, with_a(OUT, al));
                cx.ellipse(bx, by - 28.0, 37.0, 8.0, with_a(Color32::from_rgb(255, 250, 244), al));
                cx.ellipse(bx, by - 28.0, 31.0, 6.0, with_a(Color32::from_rgb(255, 252, 246), al));
                if play > 0.3 {
                    for i in 0..3 {
                        let tt = (t * 2.0 + i as f32 * 0.4) % 1.0;
                        cx.circle(bx - 16.0 + i as f32 * 12.0, by - 34.0 - tt * 22.0, 2.6 * (1.0 - tt), with_a(Color32::WHITE, al * (1.0 - tt)));
                    }
                }
            }
            Trick::MilkBottle => {
                let ang = -pose.face * 0.6;
                let bx = hx + pose.face * 30.0 * k;
                let by = hy + 8.0 * k;
                let (sn, cs) = ang.sin_cos();
                let off = |dx: f32, dy: f32| (bx + dx * cs - dy * sn, by + dx * sn + dy * cs);
                let cap = off(0.0, -30.0 * k);
                let nip = off(0.0, 30.0 * k);
                cx.ellipse_rot(bx, by, 16.0 * k * 0.6, 30.0 * k * 0.6, ang, with_a(OUT, al));
                cx.ellipse_rot(bx, by, 13.0 * k * 0.6, 27.0 * k * 0.6, ang, with_a(Color32::from_rgb(255, 252, 246), al));
                cx.ellipse_rot(bx, by + 4.0, 10.0 * k * 0.6, 20.0 * k * 0.6, ang, with_a(Color32::from_rgb(255, 244, 224), al));
                cx.rect(cap.0 - 8.0, cap.1 - 6.0, cap.0 + 8.0, cap.1 + 4.0, 3.0, with_a(Color32::from_rgb(255, 178, 196), al));
                cx.ellipse(nip.0, nip.1, 6.0, 7.0, with_a(OUT, al));
                cx.ellipse(nip.0, nip.1, 4.5, 5.5, with_a(Color32::from_rgb(255, 206, 176), al));
            }
            Trick::Knead => {
                for (i, note) in ["♪", "♫"].iter().enumerate() {
                    let tt = (t * 0.7 + i as f32 * 0.5) % 1.0;
                    let nx = x + (i as f32 * 2.0 - 1.0) * 34.0 * k;
                    let ny = gy - 50.0 * k - tt * 46.0;
                    cx.text(nx, ny, Align2::CENTER_CENTER, note, 20.0, with_a(Color32::from_rgb(150, 110, 180), al * (1.0 - tt)), true);
                }
            }
            Trick::FriendVisit => {}
            Trick::GoldenKitten => {
                let pulse = 0.6 + 0.4 * (t * 3.0).sin();
                cx.glow(x, gy - 110.0 * k, 200.0 * k, GOLD, 0.4 * al * pulse);
                for i in 0..14 {
                    let ang = i as f32 / 14.0 * TAU + t * 0.9;
                    let r = 120.0 * k + (t * 2.0 + i as f32).sin() * 18.0;
                    draw_star(cx, x + ang.cos() * r, gy - 110.0 * k + ang.sin() * r * 0.62, 6.0 + (i % 3) as f32 * 2.5, with_a(GOLD, al * 0.9));
                }
                let kgy = gy - 150.0 * k - (t * 1.6).sin() * 10.0;
                let kitten = Look { fur: Color32::from_rgb(255, 224, 140), patch: Some(Color32::from_rgb(238, 158, 62)), patch2: None, scale: k * 0.42, level: 1 };
                let kp = Pose { form: Form::Sit, eyes: Eyes::Happy, face: 1.0, walk: 0.0, moving: 0.0, hop: 0.0, wag: 1.0, t, knead: false, amt: 0.0 };
                draw_cat(cx, x, kgy, &kitten, &kp, al);
                cx.text_o(x, kgy - 120.0 * k * 0.42 - 16.0, Align2::CENTER_BOTTOM, "ลูกแมวทองคำ!", 28.0, with_a(Color32::from_rgb(255, 246, 214), al), with_a(Color32::from_rgb(150, 96, 20), al), true);
            }
        }
    }

    fn draw_hearts(&self, cx: &Cx, g: f32) {
        let k = look_for(self.level).scale;
        let (_, gy) = self.cat_screen(g);
        for h in &self.hearts {
            let u = h.life / h.max;
            let a = (1.0 - u).clamp(0.0, 1.0);
            let y = gy - 100.0 * k + h.y;
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
            let x = self.cat_x.clamp(220.0, VW - 220.0);
            let y = cx.vh * 0.45;
            cx.glow(x, y, 300.0, GOLD, 0.35 * a);
            cx.text_o(x, y, Align2::CENTER_CENTER, &format!("เลเวล {}!", self.level), 54.0, with_a(Color32::from_rgb(255, 248, 220), a), with_a(Color32::from_rgb(150, 96, 20), a), true);
            cx.text_o(x, y + 48.0, Align2::CENTER_CENTER, level_title(self.level), 30.0, with_a(Color32::from_rgb(255, 240, 200), a), with_a(Color32::from_rgb(120, 70, 30), a), true);
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
        cx.rect(x0, y0, x0 + w, y0 + h, 16.0, rgba(255, 250, 240, 0.92));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 16.0, 2.5, rgba(122, 86, 60, 0.8));
        cx.text(x0 + 16.0, y0 + 26.0, Align2::LEFT_CENTER, &self.name, 23.0, Color32::from_rgb(70, 48, 40), true);
        let lv = format!("Lv.{} {}", self.level, level_title(self.level));
        cx.text(x0 + w - 16.0, y0 + 26.0, Align2::RIGHT_CENTER, &lv, 18.0, Color32::from_rgb(190, 120, 40), true);
        let bars = [
            ("ความเหงา", self.loneliness, Color32::from_rgb(150, 190, 255), Color32::from_rgb(90, 130, 220), true),
            ("ความอิ่ม", self.fullness, Color32::from_rgb(255, 206, 130), Color32::from_rgb(232, 150, 70), false),
            ("ความผูกพัน", self.bond, Color32::from_rgb(255, 150, 190), Color32::from_rgb(226, 90, 140), false),
        ];
        for (i, (label, v, c1, c2, invert)) in bars.iter().enumerate() {
            let y = y0 + 52.0 + i as f32 * row;
            cx.text(x0 + 16.0, y + 13.0, Align2::LEFT_CENTER, label, 17.0, Color32::from_rgb(90, 64, 52), false);
            let bx0 = x0 + 116.0;
            let bx1 = x0 + w - 62.0;
            cx.rect(bx0, y + 4.0, bx1, y + 24.0, 10.0, rgba(120, 90, 70, 0.15));
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
            cx.text(x0 + w - 16.0, y + 14.0, Align2::RIGHT_CENTER, &format!("{}", v.round() as u32), 17.0, Color32::from_rgb(70, 48, 40), true);
        }
    }

    fn draw_bubble(&self, cx: &Cx) {
        let g = floor_y(cx.vh);
        let look = look_for(self.level);
        let pose = self.pose_of();
        let (x, gy) = self.cat_screen(g);
        let (hx, hy) = head_screen(x, gy, &look, &pose);
        let sleeping = pose.form == Form::Sleep;
        let mood = self.mood();
        let text = if sleeping { "zZ… ฝันถึงปลาทู" } else { mood.bubble() };
        let bx = hx.clamp(150.0, VW - 150.0);
        let by = (hy - 34.0 * look.scale).max(270.0);
        let size = cx.measure(text, 21.0, true);
        let w = size.x + 34.0;
        let h = 44.0;
        let bob = (cx.time * 1.7).sin() * 4.0;
        let col = if sleeping {
            Color32::from_rgb(224, 232, 255)
        } else {
            match mood {
                Mood::Lonely => Color32::from_rgb(214, 228, 255),
                Mood::Hungry => Color32::from_rgb(255, 232, 200),
                _ => Color32::from_rgb(255, 240, 246),
            }
        };
        cx.rect(bx - w * 0.5, by - h + bob, bx + w * 0.5, by + bob, 18.0, with_a(col, 0.95));
        cx.rect_line(bx - w * 0.5, by - h + bob, bx + w * 0.5, by + bob, 18.0, 2.0, with_a(OUT, 0.4));
        cx.poly(&[(bx - 26.0, by - 4.0 + bob), (bx - 6.0, by - 4.0 + bob), (bx - 22.0, by + 18.0 + bob)], with_a(col, 0.95));
        cx.text(bx, by - h * 0.5 + bob, Align2::CENTER_CENTER, text, 21.0, INK, true);
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
        cx.rect(x0, y0, x0 + w, y0 + h, 20.0, with_a(rgba(255, 250, 240, 0.94), al));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 20.0, 3.0, with_a(if a.trick.is_star() { GOLD } else { PINK }, al * 0.9));
        cx.text(x0 + w * 0.5, y0 + 20.0, Align2::CENTER_CENTER, title, 19.0, with_a(Color32::from_rgb(150, 90, 70), al), true);
        cx.text(x0 + w * 0.5, y0 + 50.0, Align2::CENTER_CENTER, a.trick.label(), 30.0, with_a(Color32::from_rgb(70, 48, 40), al), true);
        let who = if a.combo > 1 { format!("{}  ×{}", a.user, a.combo) } else { a.user.clone() };
        cx.text(x0 + w * 0.5, y0 + 82.0, Align2::CENTER_CENTER, &who, 21.0, with_a(Color32::from_rgb(190, 120, 40), al), true);
        if !a.gift.trim().is_empty() {
            let g = format!("{} · {} เพชร", a.gift, a.diamonds);
            cx.text(x0 + w - 16.0, y0 + 20.0, Align2::RIGHT_CENTER, &g, 15.0, with_a(Color32::from_rgb(140, 110, 90), al * 0.9), false);
        }
    }

    fn draw_guide(&self, cx: &Cx, label: &str) {
        let cols = 2;
        let rows = 5;
        let cw = 268.0;
        let rh = 32.0;
        let w = cw * cols as f32 + 26.0;
        let h = rh * rows as f32 + 46.0;
        let x0 = VW - w - 20.0;
        let y0 = 14.0;
        cx.rect(x0, y0, x0 + w, y0 + h, 14.0, rgba(255, 250, 240, 0.9));
        cx.rect_line(x0, y0, x0 + w, y0 + h, 14.0, 2.0, rgba(122, 86, 60, 0.6));
        cx.text(x0 + w * 0.5, y0 + 22.0, Align2::CENTER_CENTER, label, 18.0, Color32::from_rgb(120, 70, 50), true);
        for (i, t) in TRICKS.iter().enumerate() {
            let c = i / rows;
            let r = i % rows;
            let x = x0 + 14.0 + c as f32 * cw;
            let y = y0 + 44.0 + r as f32 * rh;
            let active = self.active.as_ref().map(|a| a.trick == *t).unwrap_or(false);
            if active {
                cx.rect(x - 6.0, y - 12.0, x + cw - 12.0, y + 22.0, 8.0, rgba(255, 170, 200, 0.35));
            }
            cx.circle(x + 7.0, y + 2.0, 6.0, if t.is_food() { Color32::from_rgb(255, 206, 130) } else { PINK });
            cx.text(x + 20.0, y, Align2::LEFT_CENTER, t.label(), 15.0, Color32::from_rgb(80, 56, 46), true);
            // ชื่อสติกเกอร์เต็ม ไม่ตัดจบ — ย่อฟอนต์อัตโนมัติถ้าล้นคอลัมน์
            let gift = t.gift();
            let mut gs = 12.5;
            while gs > 9.0 && cx.measure(gift, gs, false).x > cw - 74.0 {
                gs -= 0.5;
            }
            cx.text(x + 20.0, y + 15.0, Align2::LEFT_CENTER, gift, gs, Color32::from_rgb(150, 122, 100), false);
            cx.text(x + cw - 20.0, y + 2.0, Align2::RIGHT_CENTER, &format!("{}", t.diamonds()), 15.0, Color32::from_rgb(190, 120, 40), true);
        }
    }
}

impl Default for PetStage {
    fn default() -> Self {
        Self::new()
    }
}

// --------------------------------------------------------------------- helpers

fn prop_fade(u: f32) -> f32 {
    if u > 0.86 { 1.0 - ((u - 0.86) / 0.14).clamp(0.0, 1.0) } else { 1.0 }
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = clamp01(t);
    Color32::from_rgb(lerp(a.r() as f32, b.r() as f32, t) as u8, lerp(a.g() as f32, b.g() as f32, t) as u8, lerp(a.b() as f32, b.b() as f32, t) as u8)
}

fn toy_x(clock: f32, play: f32) -> f32 {
    PET_X + (clock * 2.6).sin() * 150.0 * play
}

/// ตำแหน่งหัวในพิกัดแมว (ยังไม่คูณสเกล/กลับด้าน) — ใช้ยึดฟองคำพูด/ของเล่นให้ติดหัว
fn head_local(form: Form, t: f32, amt: f32) -> (f32, f32, f32) {
    match form {
        Form::Sit | Form::PawUp => (10.0, -78.0, 0.0),
        Form::Stand => (32.0, -56.0, 0.0),
        Form::Loaf => (22.0, -42.0, 0.0),
        Form::Sleep => (16.0, -26.0, 0.3),
        Form::Groom => (12.0, -74.0, 0.12),
        Form::Stretch => (36.0, -26.0, 0.2 * amt),
        Form::Eat => (34.0, -22.0 + (t * 3.0).sin() * 1.5, 0.25),
        Form::Bat => (8.0, -82.0, 0.0),
    }
}

fn head_screen(x: f32, gy: f32, look: &Look, pose: &Pose) -> (f32, f32) {
    let (hx, hy, _) = head_local(pose.form, pose.t, pose.amt);
    (x + pose.face * hx * look.scale, gy - pose.hop + hy * look.scale)
}

fn ellipse_pts(cx0: f32, cy0: f32, rx: f32, ry: f32, ang: f32, n: usize) -> Vec<(f32, f32)> {
    let (sn, cs) = ang.sin_cos();
    (0..n)
        .map(|i| {
            let a = i as f32 / n as f32 * TAU;
            let (x, y) = (rx * a.cos(), ry * a.sin());
            (cx0 + x * cs - y * sn, cy0 + x * sn + y * cs)
        })
        .collect()
}

/// Sutherland–Hodgman: ตัด polygon นูนด้วย polygon นูน (ใช้ตัดลายขนให้ติดตัวแมว)
fn clip_convex(subject: &[(f32, f32)], clip: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let mut output = subject.to_vec();
    let n = clip.len();
    for i in 0..n {
        if output.len() < 3 {
            return Vec::new();
        }
        let a = clip[i];
        let b = clip[(i + 1) % n];
        let side = |p: (f32, f32)| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
        let input = std::mem::take(&mut output);
        let m = input.len();
        for j in 0..m {
            let cur = input[j];
            let prev = input[(j + m - 1) % m];
            let sc = side(cur);
            let sp = side(prev);
            if sc >= 0.0 {
                if sp < 0.0 {
                    let tt = sp / (sp - sc);
                    output.push((prev.0 + (cur.0 - prev.0) * tt, prev.1 + (cur.1 - prev.1) * tt));
                }
                output.push(cur);
            } else if sp >= 0.0 {
                let tt = sp / (sp - sc);
                output.push((prev.0 + (cur.0 - prev.0) * tt, prev.1 + (cur.1 - prev.1) * tt));
            }
        }
    }
    output
}

fn patch(cx: &Cx, body: &[(f32, f32)], ex: f32, ey: f32, rx: f32, ry: f32, ang: f32, c: Color32) {
    let e = ellipse_pts(ex, ey, rx, ry, ang, 24);
    let cl = clip_convex(&e, body);
    if cl.len() >= 3 {
        cx.poly(&cl, c);
    }
}

fn inflate_tri(tri: [(f32, f32); 3], d: f32) -> [(f32, f32); 3] {
    let cx0 = (tri[0].0 + tri[1].0 + tri[2].0) / 3.0;
    let cy0 = (tri[0].1 + tri[1].1 + tri[2].1) / 3.0;
    tri.map(|p| {
        let dx = p.0 - cx0;
        let dy = p.1 - cy0;
        let l = (dx * dx + dy * dy).sqrt().max(0.001);
        (p.0 + dx / l * d, p.1 + dy / l * d)
    })
}

/// หาง/ขา แบบเส้นกลมเนียน: วาดขอบเข้มก่อนแล้วถมสีทับ (สอง pass = เส้นขอบรวมหนาสม่ำเสมอ)
fn tube(cx: &Cx, pts: &[(f32, f32, f32)], out: Color32, fill: Color32, ow: f32) {
    if pts.len() < 2 {
        return;
    }
    let n = pts.len();
    let mut s: Vec<(f32, f32, f32)> = Vec::new();
    for i in 0..n - 1 {
        let p0 = pts[i.saturating_sub(1)];
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = pts[(i + 2).min(n - 1)];
        for j in 0..8 {
            let u = j as f32 / 8.0;
            let u2 = u * u;
            let u3 = u2 * u;
            let cr = |a: f32, b: f32, c: f32, d: f32| 0.5 * ((2.0 * b) + (-a + c) * u + (2.0 * a - 5.0 * b + 4.0 * c - d) * u2 + (-a + 3.0 * b - 3.0 * c + d) * u3);
            s.push((cr(p0.0, p1.0, p2.0, p3.0), cr(p0.1, p1.1, p2.1, p3.1), lerp(p1.2, p2.2, u)));
        }
    }
    s.push(pts[n - 1]);
    for p in &s {
        cx.circle(p.0, p.1, p.2 + ow, out);
    }
    for p in &s {
        cx.circle(p.0, p.1, p.2, fill);
    }
}

fn draw_hand(cx: &Cx, hx: f32, hy: f32, al: f32, pat: f32) {
    let sleeve = Color32::from_rgb(126, 190, 235);
    let skin = Color32::from_rgb(255, 222, 198);
    let out = with_a(OUT, al);
    // แขนสั้นจากขวา (ไม่ลากยาวทั้งจอ) พร้อมขอบเข้ม
    cx.rect(hx + 18.0, hy - 19.0, hx + 78.0, hy + 21.0, 18.0, out);
    cx.rect(hx + 20.0, hy - 16.0, hx + 76.0, hy + 18.0, 16.0, with_a(sleeve, al));
    cx.rect(hx + 18.0, hy - 19.0, hx + 34.0, hy + 21.0, 8.0, with_a(Color32::from_rgb(96, 160, 210), al));
    cx.ellipse(hx + 8.0, hy, 27.0, 20.0, out);
    cx.ellipse(hx + 8.0, hy, 24.0, 17.0, with_a(skin, al));
    for i in 0..4 {
        let fx = hx - 14.0 + i as f32 * 9.0;
        let fy = hy + 10.0 + (i as f32 * 0.9).sin() * 2.5 + pat * 3.0;
        cx.ellipse(fx, fy, 5.5, 10.0, out);
        cx.ellipse(fx, fy, 4.3, 8.6, with_a(Color32::from_rgb(255, 228, 206), al));
    }
}

// --------------------------------------------------------------------- drawing

fn draw_paw(cx: &Cx, x: f32, y: f32, r: f32, c: Color32) {
    cx.ellipse(x, y + r * 0.32, r * 0.78, r * 0.62, c);
    for i in 0..4 {
        let a = -PI * 0.78 + i as f32 * (PI * 0.52 / 3.0);
        cx.ellipse(x + a.cos() * r * 0.72, y + a.sin() * r * 0.62, r * 0.26, r * 0.32, c);
    }
}

fn sakura_blossom(cx: &Cx, x: f32, y: f32, r: f32, alpha: f32) {
    let pink = with_a(Color32::from_rgb(255, 183, 205), alpha);
    for i in 0..5 {
        let a = -PI / 2.0 + i as f32 * TAU / 5.0;
        cx.ellipse_rot(x + a.cos() * r * 0.72, y + a.sin() * r * 0.72, r * 0.5, r * 0.68, a + PI / 2.0, pink);
    }
    cx.circle(x, y, r * 0.32, with_a(Color32::from_rgb(255, 236, 170), alpha));
}

fn sakura_petal(cx: &Cx, x: f32, y: f32, r: f32, ang: f32, alpha: f32) {
    cx.ellipse_rot(x, y, r * 0.62, r, ang, with_a(Color32::from_rgb(255, 190, 210), alpha));
}

fn sparkle(cx: &Cx, x: f32, y: f32, r: f32, alpha: f32) {
    let c = with_a(Color32::from_rgb(255, 250, 230), alpha);
    cx.poly(&[(x, y - r), (x + r * 0.22, y), (x, y + r), (x - r * 0.22, y)], c);
    cx.poly(&[(x - r, y), (x, y - r * 0.22), (x + r, y), (x, y + r * 0.22)], c);
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

/// แมวสไตล์ Neko Atsume: เส้นขอบหนา merged, สีแบน, ตาจุด, ลายส้ม/สามสี
/// วาดสอง pass (ขอบเข้มทั้งตัวก่อน แล้วถมสี) เพื่อให้เส้นรอบตัวรวมเป็นเส้นเดียวหนา ๆ
fn draw_cat(cx: &Cx, x: f32, gy: f32, look: &Look, pose: &Pose, alpha: f32) {
    if alpha <= 0.02 {
        return;
    }
    let k = look.scale;
    let f = pose.face;
    let t = pose.t;
    let fur = with_a(look.fur, alpha);
    let out = with_a(OUT, alpha);
    let tail_col = with_a(look.patch.unwrap_or(look.fur), alpha);
    let far_col = with_a(shade(look.fur, 0.88), alpha);
    let ow = 2.6;
    let pt = |lx: f32, ly: f32| (x + f * lx * k, gy - pose.hop + ly * k);

    let shadow_rx = match pose.form {
        Form::Stand => 48.0,
        Form::Loaf | Form::Sleep => 42.0,
        _ => 38.0,
    };
    cx.ellipse(x, gy + 3.0, shadow_rx * k, 8.0 * k, rgba(0, 0, 0, 0.16 * alpha));

    // รูปทรงแต่ละท่า: พิกัดท้องถิ่น (หันหน้า +x, y ลบ = ขึ้น)
    let (hxl, hyl, tilt_l) = head_local(pose.form, t, pose.amt);
    let tilt = f * tilt_l;
    let hc = pt(hxl, hyl);
    let (sa, ca) = tilt.sin_cos();
    let hp = |dx: f32, dy: f32| {
        let lx = f * dx * k;
        let ly = dy * k;
        (hc.0 + lx * ca - ly * sa, hc.1 + lx * sa + ly * ca)
    };
    let wag = (t * 2.2).sin() * 6.0 * (1.0 + pose.wag);
    let sw = (pose.walk).sin() * 9.0 * pose.moving;
    let sw2 = (pose.walk + PI).sin() * 7.0 * pose.moving;
    let mut ell: Vec<(f32, f32, f32, f32, f32, Color32)> = Vec::new();
    let mut tub: Vec<(Vec<(f32, f32, f32)>, Color32)> = Vec::new();
    let mut front_tub: Vec<(Vec<(f32, f32, f32)>, Color32)> = Vec::new();
    let mut arms: Vec<(Vec<(f32, f32, f32)>, Color32)> = Vec::new();
    let mut front_ell: Vec<(f32, f32, f32, f32, Color32)> = Vec::new();
    let body_patch: Vec<(f32, f32, f32, f32)>;
    let body_idx: usize;

    match pose.form {
        Form::Sit | Form::PawUp | Form::Groom => {
            tub.push((vec![(-26.0, -10.0, 7.0), (-42.0, -6.0, 6.0), (-48.0, -20.0, 5.0), (-40.0, -34.0 + wag, 3.5)], tail_col));
            ell.push((-16.0, -16.0, 17.0, 15.0, 0.0, fur));
            body_idx = ell.len();
            ell.push((0.0, -30.0, 30.0, 28.0, 0.0, fur));
            if pose.form == Form::Groom {
                tub.push((vec![(24.0, -24.0, 5.5), (25.0, -4.0, 5.0)], fur));
                ell.push((25.0, -3.0, 7.0, 4.5, 0.0, fur));
                arms.push((vec![(16.0, -30.0, 5.5), (19.0, -50.0, 5.0), (15.0, -63.0, 4.5)], fur));
                front_ell.push((15.0, -65.0, 6.0, 5.0, fur));
            } else {
                tub.push((vec![(14.0, -26.0, 5.5), (15.0, -4.0, 5.0)], fur));
                tub.push((vec![(24.0, -24.0, 5.5), (25.0, -4.0, 5.0)], fur));
                ell.push((15.0, -3.0, 7.0, 4.5, 0.0, fur));
                ell.push((25.0, -3.0, 7.0, 4.5, 0.0, fur));
            }
            if pose.form == Form::PawUp {
                let wave = (t * 6.0).sin() * 3.0;
                arms.push((vec![(18.0, -34.0, 5.5), (28.0, -60.0, 5.0), (32.0 + wave, -84.0, 4.5)], fur));
                front_ell.push((32.0 + wave, -88.0, 7.0, 6.0, fur));
            }
            body_patch = vec![(-12.0, -38.0, 16.0, 12.0), (14.0, -22.0, 11.0, 9.0)];
        }
        Form::Stand => {
            tub.push((vec![(-34.0, -36.0, 6.0), (-48.0, -44.0, 5.0), (-52.0, -58.0 + wag, 3.0)], tail_col));
            tub.push((vec![(18.0, -28.0, 5.0), (18.0 + sw2, -2.0, 4.5)], far_col));
            tub.push((vec![(-18.0, -28.0, 5.0), (-18.0 - sw2, -2.0, 4.5)], far_col));
            body_idx = ell.len();
            ell.push((0.0, -34.0, 36.0, 20.0, 0.0, fur));
            tub.push((vec![(24.0, -28.0, 5.5), (24.0 + sw, -2.0, 5.0)], fur));
            tub.push((vec![(-24.0, -28.0, 5.5), (-24.0 - sw, -2.0, 5.0)], fur));
            ell.push((24.0 + sw, -2.0, 6.5, 4.0, 0.0, fur));
            ell.push((-24.0 - sw, -2.0, 6.5, 4.0, 0.0, fur));
            body_patch = vec![(-10.0, -40.0, 16.0, 12.0), (16.0, -28.0, 12.0, 9.0)];
        }
        Form::Loaf => {
            body_idx = ell.len();
            ell.push((0.0, -17.0, 36.0, 17.0, 0.0, fur));
            front_tub.push((vec![(-34.0, -8.0, 6.0), (-40.0, -1.0, 5.0), (-18.0, 3.0, 4.5), (12.0, 3.0, 3.5)], tail_col));
            if pose.knead {
                let p1 = ((t * 3.4).sin() * 0.5 + 0.5) * 4.0;
                let p2 = ((t * 3.4 + PI).sin() * 0.5 + 0.5) * 4.0;
                arms.push((vec![(24.0, -10.0, 5.0), (26.0, -2.0 + p1, 4.5)], fur));
                arms.push((vec![(32.0, -10.0, 5.0), (34.0, -2.0 + p2, 4.5)], fur));
            }
            body_patch = vec![(-12.0, -22.0, 15.0, 10.0), (14.0, -12.0, 10.0, 7.0)];
        }
        Form::Sleep => {
            let breath = (t * 1.3).sin() * 0.8;
            body_idx = ell.len();
            ell.push((0.0, -20.0, 34.0, 21.0 + breath, 0.0, fur));
            front_tub.push((vec![(-30.0, -6.0, 6.0), (-36.0, 2.0, 5.0), (-10.0, 6.0, 4.5), (18.0, 4.0, 3.5), (30.0, -2.0, 3.0)], tail_col));
            body_patch = vec![(-10.0, -28.0, 15.0, 11.0)];
        }
        Form::Stretch => {
            let a = pose.amt;
            tub.push((vec![(-32.0, -34.0, 6.0), (-44.0, -48.0, 5.0), (-40.0, -60.0 + wag, 3.0)], tail_col));
            body_idx = ell.len();
            ell.push((-4.0, -30.0 + 6.0 * a, 32.0, 22.0, 0.3 * a, fur));
            tub.push((vec![(-22.0, -16.0, 5.5), (-24.0, -2.0, 5.0)], fur));
            tub.push((vec![(18.0, -14.0, 5.0), (34.0, -2.0, 4.5)], fur));
            tub.push((vec![(24.0, -12.0, 5.0), (40.0, -2.0, 4.5)], fur));
            ell.push((-24.0, -2.0, 7.0, 4.5, 0.0, fur));
            ell.push((36.0, -2.0, 7.0, 4.5, 0.0, fur));
            ell.push((42.0, -2.0, 7.0, 4.5, 0.0, fur));
            body_patch = vec![(-14.0, -36.0, 15.0, 11.0)];
        }
        Form::Eat => {
            tub.push((vec![(-26.0, -10.0, 7.0), (-42.0, -6.0, 6.0), (-48.0, -20.0, 5.0), (-40.0, -34.0 + wag, 3.5)], tail_col));
            ell.push((-14.0, -16.0, 16.0, 14.0, 0.0, fur));
            body_idx = ell.len();
            ell.push((2.0, -28.0, 30.0, 26.0, 0.12, fur));
            tub.push((vec![(16.0, -24.0, 5.5), (18.0, -2.0, 5.0)], fur));
            tub.push((vec![(26.0, -22.0, 5.5), (28.0, -2.0, 5.0)], fur));
            ell.push((18.0, -2.0, 7.0, 4.5, 0.0, fur));
            ell.push((28.0, -2.0, 7.0, 4.5, 0.0, fur));
            body_patch = vec![(-10.0, -36.0, 15.0, 11.0)];
        }
        Form::Bat => {
            tub.push((vec![(-26.0, -12.0, 7.0), (-40.0, -8.0, 6.0), (-46.0, -22.0, 5.0), (-38.0, -36.0 + wag, 3.5)], tail_col));
            body_idx = ell.len();
            ell.push((-2.0, -34.0, 27.0, 30.0, 0.0, fur));
            ell.push((-12.0, -2.0, 8.0, 4.5, 0.0, fur));
            ell.push((10.0, -2.0, 8.0, 4.5, 0.0, fur));
            let a1 = -0.9 + (t * 5.2).sin() * 0.5;
            let a2 = -0.9 + (t * 5.2 + PI).sin() * 0.5;
            let p1 = (14.0 + a1.cos() * 26.0, -52.0 + a1.sin() * 26.0);
            let p2 = (22.0 + a2.cos() * 26.0, -50.0 + a2.sin() * 26.0);
            arms.push((vec![(14.0, -52.0, 5.5), (p1.0, p1.1, 4.5)], fur));
            arms.push((vec![(22.0, -50.0, 5.5), (p2.0, p2.1, 4.5)], fur));
            front_ell.push((p1.0, p1.1, 6.0, 5.5, fur));
            front_ell.push((p2.0, p2.1, 6.0, 5.5, fur));
            body_patch = vec![(-12.0, -44.0, 15.0, 12.0), (12.0, -24.0, 10.0, 8.0)];
        }
    }
    // หัว.push ท้ายสุดเสมอ (วาดทับโคนหู)
    let head_rx = match pose.form {
        Form::Stand => 24.0,
        Form::Loaf => 25.0,
        Form::Sleep => 22.0,
        Form::Stretch | Form::Eat => 24.0,
        _ => 27.0,
    };
    let head_ry = head_rx - 3.0;
    ell.push((hxl, hyl, head_rx, head_ry, tilt_l, fur));

    // หู: สามเหลี่ยมเตี้ยฐานกว้าง glued เข้ากับหัว (หมุนตาม tilt), หูไกลใช้สีลาย
    let droop = if pose.eyes == Eyes::Sad { 1.0 } else { 0.0 };
    let twitch = if pose.wag > 0.5 { (t * 3.0).sin() * 1.5 } else { 0.0 };
    let ear_far = [(-8.0, -14.0), (-26.0, -10.0), (-24.0 - 5.0 * droop, -34.0 + 10.0 * droop + twitch)];
    let ear_near = [(8.0, -14.0), (26.0, -10.0), (24.0 + 5.0 * droop, -34.0 + 10.0 * droop - twitch)];
    let ear_far_col = with_a(look.patch.unwrap_or(look.fur), alpha);

    // pass 1: ขอบเข้มทั้งหมด
    for (pts, _) in &tub {
        let sp: Vec<(f32, f32, f32)> = pts.iter().map(|p| { let s = pt(p.0, p.1); (s.0, s.1, p.2 * k) }).collect();
        tube(cx, &sp, out, out, ow * k);
    }
    for e in &ell {
        let c = pt(e.0, e.1);
        cx.ellipse_rot(c.0, c.1, (e.2 + ow) * k, (e.3 + ow) * k, f * e.4, out);
    }
    for tri in [&ear_far, &ear_near] {
        let sp = tri.map(|p| hp(p.0, p.1));
        cx.poly(&inflate_tri(sp, 3.2 * k), out);
    }
    // pass 2: ถมสี (หาง/ขาไกล → ตัว → ขาใกล้ → หู → หัว)
    for (pts, col) in &tub {
        let sp: Vec<(f32, f32, f32)> = pts.iter().map(|p| { let s = pt(p.0, p.1); (s.0, s.1, p.2 * k) }).collect();
        tube(cx, &sp, out, *col, ow * k);
    }
    for e in ell.iter().take(ell.len() - 1) {
        let c = pt(e.0, e.1);
        cx.ellipse_rot(c.0, c.1, e.2 * k, e.3 * k, f * e.4, e.5);
    }
    for (tri, col) in [(&ear_far, ear_far_col), (&ear_near, fur)] {
        let sp = tri.map(|p| hp(p.0, p.1));
        cx.poly(&sp, col);
        let inner = sp.map(|p| {
            let cx0 = (sp[0].0 + sp[1].0 + sp[2].0) / 3.0;
            let cy0 = (sp[0].1 + sp[1].1 + sp[2].1) / 3.0;
            (cx0 + (p.0 - cx0) * 0.55, cy0 + (p.1 - cy0) * 0.55 + 2.0 * k)
        });
        cx.poly(&inner, with_a(PINK, alpha * 0.75));
    }
    {
        let e = ell.last().unwrap();
        let c = pt(e.0, e.1);
        cx.ellipse_rot(c.0, c.1, e.2 * k, e.3 * k, f * e.4, e.5);
    }
    // ลายขนตัดตามตัว/หัว
    let be = ell[body_idx];
    let bc = pt(be.0, be.1);
    let body_poly = ellipse_pts(bc.0, bc.1, be.2 * k, be.3 * k, f * be.4, 32);
    if let Some(pc) = look.patch {
        if let Some(bp) = body_patch.first() {
            let c = pt(bp.0, bp.1);
            patch(cx, &body_poly, c.0, c.1, bp.2 * k, bp.3 * k, 0.0, with_a(pc, alpha));
        }
    }
    if let Some(pc2) = look.patch2 {
        if let Some(bp) = body_patch.get(1) {
            let c = pt(bp.0, bp.1);
            patch(cx, &body_poly, c.0, c.1, bp.2 * k, bp.3 * k, 0.0, with_a(pc2, alpha));
        }
    }
    let head_poly = ellipse_pts(hc.0, hc.1, head_rx * k, head_ry * k, tilt, 32);
    if let Some(pc) = look.patch {
        let c = hp(-12.0, -14.0);
        patch(cx, &head_poly, c.0, c.1, 15.0 * k, 12.0 * k, tilt, with_a(pc, alpha));
    }
    if let Some(pc2) = look.patch2 {
        let c = hp(14.0, -18.0);
        patch(cx, &head_poly, c.0, c.1, 10.0 * k, 8.0 * k, tilt, with_a(pc2, alpha));
    }
    // หางพันด้านหน้า (ท่า loaf/นอน) วาดทับตัว
    for (pts, col) in &front_tub {
        let sp: Vec<(f32, f32, f32)> = pts.iter().map(|p| { let s = pt(p.0, p.1); (s.0, s.1, p.2 * k) }).collect();
        tube(cx, &sp, out, *col, ow * k);
    }

    // หน้า: ยึดกรอบหัว (เอียงตาม tilt)
    for s in [-1.0, 1.0] {
        let e = hp(s * 11.0, -2.0);
        match pose.eyes {
            Eyes::Dot => {
                cx.ellipse(e.0, e.1, 3.2 * k, 4.2 * k, out);
                cx.circle(e.0 + f * 1.0 * k, e.1 - 1.5 * k, 1.2 * k, with_a(Color32::WHITE, alpha * 0.9));
            }
            Eyes::Wide => {
                cx.ellipse(e.0, e.1, 4.5 * k, 5.5 * k, out);
                cx.circle(e.0 + f * 1.4 * k, e.1 - 1.8 * k, 1.8 * k, with_a(Color32::WHITE, alpha * 0.95));
            }
            Eyes::Happy => {
                let arc = arc_pts(e.0, e.1 + 1.0 * k, 5.0 * k, 4.5 * k, PI, TAU, 8);
                cx.polyline(&arc, 2.6 * k, out);
            }
            Eyes::Closed => {
                let arc = arc_pts(e.0, e.1 - 1.0 * k, 5.0 * k, 4.0 * k, 0.0, PI, 8);
                cx.polyline(&arc, 2.4 * k, out);
            }
            Eyes::Sad => {
                cx.ellipse(e.0, e.1 + 1.0 * k, 3.0 * k, 3.8 * k, out);
                cx.circle(e.0 + f * 1.0 * k, e.1 - 0.5 * k, 1.1 * k, with_a(Color32::WHITE, alpha * 0.9));
                let b1 = hp(s * 11.0 - 4.0, -10.0);
                let b2 = hp(s * 11.0 + 5.0, -7.0);
                cx.line(b1, b2, 2.0 * k, out);
                if s > 0.0 {
                    let td = (t * 0.9) % 1.6;
                    if td < 1.0 {
                        let tear = hp(16.0, 6.0 + td * 16.0);
                        cx.circle(tear.0, tear.1, 2.6 * k * (1.0 - td * 0.5), with_a(Color32::from_rgb(150, 210, 255), alpha * (1.0 - td)));
                    }
                }
            }
        }
    }
    // แก้มชมพู + จมูก + ปาก ω
    for s in [-1.0, 1.0] {
        let b = hp(s * 17.0, 8.0);
        cx.ellipse(b.0, b.1, 4.5 * k, 2.5 * k, with_a(Color32::from_rgb(255, 140, 160), alpha * 0.35));
    }
    let n1 = hp(-3.0, 5.0);
    let n2 = hp(3.0, 5.0);
    let n3 = hp(0.0, 8.5);
    cx.poly(&[n1, n2, n3], with_a(PINK, alpha));
    if pose.form == Form::Eat && pose.eyes == Eyes::Happy {
        let m = hp(0.0, 12.0);
        cx.ellipse(m.0, m.1, 4.0 * k, 3.5 * k, out);
        let tg = hp(0.0, 13.5);
        cx.ellipse(tg.0, tg.1, 2.5 * k, 1.8 * k, with_a(Color32::from_rgb(255, 130, 150), alpha));
    } else {
        let m: Vec<(f32, f32)> = [(-7.0, 10.0), (-3.5, 13.0), (0.0, 10.5), (3.5, 13.0), (7.0, 10.0)].iter().map(|p| hp(p.0, p.1)).collect();
        cx.polyline(&m, 2.0 * k, out);
    }
    if pose.form == Form::Groom && (t * 6.0).sin() > 0.2 {
        let tg = hp(2.0, 11.0);
        cx.ellipse(tg.0, tg.1, 2.5 * k, 1.8 * k, with_a(Color32::from_rgb(255, 130, 150), alpha));
    }
    for s in [-1.0, 1.0] {
        let w1a = hp(s * 16.0, 6.0);
        let w1b = hp(s * 34.0, 2.0);
        let w2a = hp(s * 16.0, 9.0);
        let w2b = hp(s * 34.0, 11.0);
        cx.line(w1a, w1b, 1.5 * k, with_a(OUT, alpha * 0.55));
        cx.line(w2a, w2b, 1.5 * k, with_a(OUT, alpha * 0.55));
    }

    // แขน/อุ้งเท้าที่ยก (วาดทับหัว)
    for (pts, col) in &arms {
        let sp: Vec<(f32, f32, f32)> = pts.iter().map(|p| { let s = pt(p.0, p.1); (s.0, s.1, p.2 * k) }).collect();
        tube(cx, &sp, out, *col, ow * k);
    }
    for e in &front_ell {
        let c = pt(e.0, e.1);
        cx.ellipse(c.0, c.1, (e.2 + ow * 0.7) * k, (e.3 + ow * 0.7) * k, out);
        cx.ellipse(c.0, c.1, e.2 * k, e.3 * k, e.4);
    }
    if pose.form == Form::PawUp {
        let wave = (t * 6.0).sin() * 3.0;
        let pad = pt(32.0 + wave, -88.0);
        cx.circle(pad.0, pad.1 + 1.0 * k, 2.6 * k, with_a(PINK, alpha * 0.85));
        for i in 0..3 {
            cx.circle(pad.0 + (i as f32 - 1.0) * 3.0 * k, pad.1 - 2.6 * k, 1.3 * k, with_a(PINK, alpha * 0.85));
        }
    }

    // ปลอกคอ + กระดิ่ง (เลเวล 2+), โบว์ (3+), มงกุฎ (5)
    if look.level >= 2 && pose.form != Form::Sleep {
        let c = hp(0.0, 21.0);
        cx.ellipse_rot(c.0, c.1, 16.5 * k, 6.0 * k, tilt, out);
        cx.ellipse_rot(c.0, c.1, 15.0 * k, 4.5 * k, tilt, with_a(Color32::from_rgb(226, 86, 120), alpha));
        let bell = hp(0.0, 27.0);
        cx.circle(bell.0, bell.1, 5.2 * k, out);
        cx.circle(bell.0, bell.1, 4.0 * k, with_a(GOLD, alpha));
        cx.circle(bell.0, bell.1 + 0.8 * k, 1.4 * k, with_a(Color32::from_rgb(190, 140, 40), alpha));
    }
    if look.level >= 3 {
        let b = hp(20.0, -28.0);
        let t1 = [hp(20.0, -28.0), hp(9.0, -35.0), hp(9.0, -21.0)];
        let t2 = [hp(20.0, -28.0), hp(31.0, -35.0), hp(31.0, -21.0)];
        cx.poly(&inflate_tri(t1, 1.6 * k), out);
        cx.poly(&inflate_tri(t2, 1.6 * k), out);
        cx.poly(&t1, with_a(Color32::from_rgb(255, 120, 170), alpha));
        cx.poly(&t2, with_a(Color32::from_rgb(255, 120, 170), alpha));
        cx.circle(b.0, b.1, 3.0 * k, with_a(Color32::from_rgb(255, 200, 220), alpha));
    }
    if look.level >= 5 {
        let gold = with_a(Color32::from_rgb(255, 158, 44), alpha);
        let edge = with_a(Color32::from_rgb(146, 94, 20), alpha);
        let band = [hp(-15.0, -40.0), hp(15.0, -40.0), hp(15.0, -33.0), hp(-15.0, -33.0)];
        cx.poly(&band, edge);
        let band2 = [hp(-13.5, -39.0), hp(13.5, -39.0), hp(13.5, -34.0), hp(-13.5, -34.0)];
        cx.poly(&band2, gold);
        for (dx, tip) in [(-10.0, -50.0), (0.0, -55.0), (10.0, -50.0)] {
            let to = [hp(dx - 5.0, -40.0), hp(dx + 5.0, -40.0), hp(dx, tip)];
            let ti = [hp(dx - 3.2, -40.0), hp(dx + 3.2, -40.0), hp(dx, tip + 2.5)];
            cx.poly(&to, edge);
            cx.poly(&ti, gold);
        }
        for (dx, c) in [(-8.0, Color32::from_rgb(255, 110, 150)), (0.0, Color32::from_rgb(150, 220, 255)), (8.0, Color32::from_rgb(255, 110, 150))] {
            let gem = hp(dx, -36.5);
            cx.circle(gem.0, gem.1, 1.8 * k, with_a(c, alpha));
        }
        let gl = hp(0.0, -44.0);
        cx.glow(gl.0, gl.1, 34.0 * k, GOLD, 0.25 * alpha);
    }
    // zZ ลอยตอนนอน
    if pose.form == Form::Sleep && alpha > 0.5 {
        for i in 0..3 {
            let u = (t * 0.45 + i as f32 * 0.33) % 1.0;
            let z = hp(26.0 + u * 14.0, -24.0 - u * 30.0);
            cx.text(z.0, z.1, Align2::CENTER_CENTER, if i == 0 { "z" } else { "Z" }, (7.0 + u * 5.0) * k * 0.9, with_a(Color32::from_rgb(110, 100, 140), alpha * (1.0 - u)), true);
        }
    }
}#[cfg(test)]
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

    #[test]
    fn idle_cat_roams_the_room() {
        let mut s = stage();
        s.set_tuning("ทดสอบ", 20.0, 0.0, 0.0, 5, 30, 4.0, 14.0);
        let mut acts = std::collections::BTreeSet::new();
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..1200 {
            s.tick(0.1);
            acts.insert(s.activity().key());
            lo = lo.min(s.cat_x());
            hi = hi.max(s.cat_x());
        }
        assert!(acts.len() >= 2, "cat should switch activities, got {:?}", acts);
        assert!(hi - lo > 60.0, "cat should walk around, range={}", hi - lo);
        assert!(lo >= WALK_MIN - 1.0 && hi <= WALK_MAX + 1.0, "out of room: {}..{}", lo, hi);
    }

    #[test]
    fn hungry_cat_goes_to_the_bowl() {
        let mut s = stage();
        s.set_tuning("ทดสอบ", 20.0, 0.0, 0.0, 5, 30, 4.0, 14.0);
        s.fullness = 10.0;
        for _ in 0..200 {
            s.tick(0.1);
        }
        assert!((s.cat_x() - BOWL_X).abs() < 160.0, "hungry cat should be near the bowl, x={}", s.cat_x());
    }

    #[test]
    fn wand_chase_makes_the_cat_follow_the_toy() {
        let mut s = stage();
        s.cat_x = 300.0;
        s.enqueue(inn(Trick::WandChase, "ก"));
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for _ in 0..50 {
            s.tick(0.1);
            lo = lo.min(s.cat_x());
            hi = hi.max(s.cat_x());
        }
        assert!(hi - lo > 60.0, "cat should chase the wand, range={}", hi - lo);
    }
}