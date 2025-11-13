//! L0: geometry, colour and key units.
//!
//! Layout geometry is in key units (u): 1u is the width of a letter key.

use serde::{Deserialize, Serialize};

/// Overlap smaller than this (in u) counts as touching, not overlapping.
pub const EPSILON: f32 = 1e-4;

/// Axis-aligned rectangle; `x`/`y` is the top-left corner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// Finite coordinates and a positive size.
    pub fn is_valid(&self) -> bool {
        [self.x, self.y, self.w, self.h]
            .iter()
            .all(|v| v.is_finite())
            && self.w > 0.0
            && self.h > 0.0
    }

    /// True if the interiors overlap by more than [`EPSILON`]; shared edges don't count.
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.x < other.right() - EPSILON
            && other.x < self.right() - EPSILON
            && self.y < other.bottom() - EPSILON
            && other.y < self.bottom() - EPSILON
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    /// Smallest rectangle containing both.
    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Rect::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn touching_is_not_overlapping() {
        let a = Rect::new(0.0, 0.0, 1.0, 1.0);
        assert!(!a.overlaps(&Rect::new(1.0, 0.0, 1.0, 1.0)));
        assert!(!a.overlaps(&Rect::new(0.0, 1.0, 1.0, 1.0)));
        assert!(a.overlaps(&Rect::new(0.5, 0.5, 1.0, 1.0)));
    }

    #[test]
    fn validity() {
        assert!(Rect::new(0.0, 0.0, 1.0, 1.0).is_valid());
        assert!(!Rect::new(0.0, 0.0, 0.0, 1.0).is_valid());
        assert!(!Rect::new(f32::NAN, 0.0, 1.0, 1.0).is_valid());
        assert!(!Rect::new(0.0, f32::INFINITY, 1.0, 1.0).is_valid());
    }

    fn rect() -> impl Strategy<Value = Rect> {
        (-50f32..50.0, -50f32..50.0, 0.01f32..10.0, 0.01f32..10.0)
            .prop_map(|(x, y, w, h)| Rect::new(x, y, w, h))
    }

    proptest! {
        #[test]
        fn overlap_is_symmetric(a in rect(), b in rect()) {
            prop_assert_eq!(a.overlaps(&b), b.overlaps(&a));
        }

        #[test]
        fn union_contains_both(a in rect(), b in rect()) {
            // Float rounding: x + (max - x) may miss max by an ulp.
            let u = a.union(&b);
            for r in [a, b] {
                prop_assert!(u.x <= r.x && u.y <= r.y);
                prop_assert!(u.right() >= r.right() - EPSILON && u.bottom() >= r.bottom() - EPSILON);
            }
        }
    }
}
