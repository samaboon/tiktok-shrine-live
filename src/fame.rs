//! Session fame: per-viewer gift totals (leaderboard) + a cumulative hype level.
//! Lives only for the running session; nothing is persisted or sent anywhere.

use std::collections::BTreeMap;

/// Stable FNV-1a hash so an avatar URL maps to the same cache/texture id every run.
pub fn stable_hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[derive(Clone, Debug, Default)]
pub struct UserStat {
    pub diamonds: u64,
    pub gifts: u64,
    /// hash of the viewer's avatar URL (0 = none yet -> procedural fallback)
    pub avatar_id: u64,
}

#[derive(Clone, Debug)]
pub struct Fame {
    users: BTreeMap<String, UserStat>,
    /// hype xp inside the current level
    pub xp: u64,
    pub level: u32,
    /// cumulative diamonds this session
    pub total: u64,
}

impl Default for Fame {
    fn default() -> Self {
        Self { users: BTreeMap::new(), xp: 0, level: 1, total: 0 }
    }
}

impl Fame {
    pub fn new() -> Self {
        Self::default()
    }

    /// diamonds needed to leave the current level (grows with level)
    pub fn need(&self) -> u64 {
        300 * self.level as u64
    }

    pub fn progress(&self) -> f32 {
        (self.xp as f32 / self.need() as f32).clamp(0.0, 1.0)
    }

    pub fn record(&mut self, user: &str, diamonds: u64, avatar_id: u64) {
        let e = self.users.entry(user.to_string()).or_default();
        e.diamonds = e.diamonds.saturating_add(diamonds);
        e.gifts += 1;
        if avatar_id != 0 {
            e.avatar_id = avatar_id;
        }
        self.total = self.total.saturating_add(diamonds);
        self.xp = self.xp.saturating_add(diamonds);
        while self.xp >= self.need() {
            self.xp -= self.need();
            self.level += 1;
        }
    }

    pub fn avatar_of(&self, user: &str) -> u64 {
        self.users.get(user).map(|u| u.avatar_id).unwrap_or(0)
    }

    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// top n viewers by diamonds (ties broken by name for a stable order)
    pub fn top(&self, n: usize) -> Vec<(&str, &UserStat)> {
        let mut v: Vec<(&str, &UserStat)> = self.users.iter().map(|(k, u)| (k.as_str(), u)).collect();
        v.sort_by(|a, b| b.1.diamonds.cmp(&a.1.diamonds).then_with(|| a.0.cmp(b.0)));
        v.truncate(n);
        v
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_accumulates_per_user() {
        let mut f = Fame::new();
        f.record("เอ", 10, 0);
        f.record("เอ", 5, 0);
        f.record("บี", 100, 0);
        assert_eq!(f.user_count(), 2);
        assert_eq!(f.total, 115);
        let top = f.top(5);
        assert_eq!(top[0].0, "บี");
        assert_eq!(top[0].1.diamonds, 100);
        assert_eq!(top[1].0, "เอ");
        assert_eq!(top[1].1.diamonds, 15);
        assert_eq!(top[1].1.gifts, 2);
    }

    #[test]
    fn top_is_capped_and_sorted() {
        let mut f = Fame::new();
        for i in 0..10 {
            f.record(&format!("u{i}"), (i * 7) as u64, 0);
        }
        let top = f.top(3);
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].0, "u9");
        assert!(top[0].1.diamonds >= top[1].1.diamonds);
        assert!(top[1].1.diamonds >= top[2].1.diamonds);
    }

    #[test]
    fn hype_level_crosses_thresholds() {
        let mut f = Fame::new();
        assert_eq!(f.level, 1);
        assert_eq!(f.need(), 300);
        f.record("x", 299, 0);
        assert_eq!(f.level, 1);
        assert!((f.progress() - 299.0 / 300.0).abs() < 1e-6);
        f.record("x", 1, 0);
        assert_eq!(f.level, 2);
        assert_eq!(f.xp, 0);
        assert_eq!(f.need(), 600);
        // a big gift can skip several levels at once
        f.record("x", 600 + 900 + 50, 0);
        assert_eq!(f.level, 4);
        assert_eq!(f.xp, 50);
        assert!(f.progress() >= 0.0 && f.progress() <= 1.0);
    }

    #[test]
    fn avatar_id_is_remembered_and_hash_is_stable() {
        let mut f = Fame::new();
        let id = stable_hash("https://example.com/a.webp");
        assert_ne!(id, 0);
        assert_eq!(id, stable_hash("https://example.com/a.webp"));
        assert_ne!(id, stable_hash("https://example.com/b.webp"));
        f.record("เอ", 1, id);
        f.record("เอ", 1, 0); // later gift without url keeps the known avatar
        assert_eq!(f.avatar_of("เอ"), id);
        assert_eq!(f.avatar_of(" nobody "), 0);
    }

    #[test]
    fn reset_clears_everything() {
        let mut f = Fame::new();
        f.record("x", 5000, 7);
        f.reset();
        assert_eq!(f.level, 1);
        assert_eq!(f.total, 0);
        assert_eq!(f.user_count(), 0);
    }
}