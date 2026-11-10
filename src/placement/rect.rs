//! A screen rectangle in physical pixels (left, top, right, bottom) and the few operations placement needs on it.

#[cfg(windows)]
use windows::Win32::Foundation::RECT;

#[derive(Clone, Copy, Debug, Default)]
pub struct R {
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
}

impl R {
    pub fn w(&self) -> i32 {
        self.r - self.l
    }
    pub fn h(&self) -> i32 {
        self.b - self.t
    }
    pub fn area(&self) -> i64 {
        self.w().max(0) as i64 * self.h().max(0) as i64
    }
    pub fn overlap(&self, o: &R) -> i64 {
        let w = (self.r.min(o.r) - self.l.max(o.l)).max(0) as i64;
        let h = (self.b.min(o.b) - self.t.max(o.t)).max(0) as i64;
        w * h
    }
    #[cfg(windows)]
    pub(super) fn from(r: RECT) -> R {
        R { l: r.left, t: r.top, r: r.right, b: r.bottom }
    }
    /// Gap between two rects (0 when touching/overlapping).
    pub(super) fn gap(&self, o: &R) -> f64 {
        let dx = (o.l - self.r).max(self.l - o.r).max(0) as f64;
        let dy = (o.t - self.b).max(self.t - o.b).max(0) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}
