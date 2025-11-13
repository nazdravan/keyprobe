//! Keyboard layouts: which physical keys a board has, where they sit, what they are labelled.

use std::collections::BTreeSet;

use kp_geom::Rect;
use serde::{Deserialize, Serialize};

use crate::{KeyCode, MAX_KEYS};

/// Longest accepted key label, in chars.
pub const MAX_LABEL_LEN: usize = 16;
/// Largest accepted board extent, in key units.
pub const MAX_EXTENT: f32 = 64.0;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LayoutError {
    #[error("layout has no keys")]
    Empty,
    #[error("layout has {0} keys (limit {MAX_KEYS})")]
    TooManyKeys(usize),
    #[error("{0} appears more than once")]
    DuplicateCode(KeyCode),
    #[error("{0} has an invalid rectangle (non-finite, non-positive, or beyond {MAX_EXTENT}u)")]
    BadRect(KeyCode),
    #[error("{a} overlaps {b}")]
    Overlap { a: KeyCode, b: KeyCode },
    #[error("{0} has a label longer than {MAX_LABEL_LEN} chars")]
    LongLabel(KeyCode),
}

/// One key on the board.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyCap {
    pub code: KeyCode,
    /// Position and size in key units.
    pub rect: Rect,
    /// Main legend (`A`, `1`, `Enter`).
    pub label: String,
    /// Legend printed above the main one, typed with Shift (`!` on `1`); empty if none.
    pub shift_label: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    /// Stable id, the value of `View::layout` (`ansi-104`).
    pub name: String,
    /// Human-readable name for menus.
    pub title: String,
    pub keys: Vec<KeyCap>,
}

impl Layout {
    /// Checks the invariants the renderer relies on: some keys, at most [`MAX_KEYS`], unique
    /// codes, finite in-bounds rectangles that don't overlap, short labels.
    pub fn validate(&self) -> Result<(), LayoutError> {
        if self.keys.is_empty() {
            return Err(LayoutError::Empty);
        }
        if self.keys.len() > MAX_KEYS {
            return Err(LayoutError::TooManyKeys(self.keys.len()));
        }
        let mut seen = BTreeSet::new();
        for key in &self.keys {
            if !seen.insert(&key.code) {
                return Err(LayoutError::DuplicateCode(key.code.clone()));
            }
            let r = key.rect;
            let in_bounds =
                r.x >= 0.0 && r.y >= 0.0 && r.right() <= MAX_EXTENT && r.bottom() <= MAX_EXTENT;
            if !r.is_valid() || !in_bounds {
                return Err(LayoutError::BadRect(key.code.clone()));
            }
            if key.label.chars().count() > MAX_LABEL_LEN
                || key.shift_label.chars().count() > MAX_LABEL_LEN
            {
                return Err(LayoutError::LongLabel(key.code.clone()));
            }
        }
        // O(n²) is fine: n <= MAX_KEYS.
        for (i, a) in self.keys.iter().enumerate() {
            for b in self.keys.iter().skip(i + 1) {
                if a.rect.overlaps(&b.rect) {
                    return Err(LayoutError::Overlap {
                        a: a.code.clone(),
                        b: b.code.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Bounding box of all keys (`None` for an empty layout).
    pub fn bounds(&self) -> Option<Rect> {
        let mut rects = self.keys.iter().map(|k| k.rect);
        let first = rects.next()?;
        Some(rects.fold(first, |acc, r| acc.union(&r)))
    }

    pub fn key(&self, code: &KeyCode) -> Option<&KeyCap> {
        self.keys.iter().find(|k| &k.code == code)
    }

    /// The key under a point in key units, for mouse hit-testing.
    pub fn key_at(&self, x: f32, y: f32) -> Option<&KeyCap> {
        self.keys.iter().find(|k| k.rect.contains(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn cap(code: &str, x: f32, y: f32, w: f32) -> KeyCap {
        KeyCap {
            code: KeyCode::new(code).unwrap(),
            rect: Rect::new(x, y, w, 1.0),
            label: code.to_owned(),
            shift_label: String::new(),
        }
    }

    fn layout(keys: Vec<KeyCap>) -> Layout {
        Layout {
            name: "t".into(),
            title: "T".into(),
            keys,
        }
    }

    #[test]
    fn validate_catches_each_problem() {
        assert_eq!(layout(vec![]).validate(), Err(LayoutError::Empty));
        assert!(
            layout(vec![cap("A", 0.0, 0.0, 1.0), cap("B", 1.0, 0.0, 1.0)])
                .validate()
                .is_ok()
        );
        assert!(matches!(
            layout(vec![cap("A", 0.0, 0.0, 1.0), cap("A", 2.0, 0.0, 1.0)]).validate(),
            Err(LayoutError::DuplicateCode(_))
        ));
        assert!(matches!(
            layout(vec![cap("A", 0.0, 0.0, 1.5), cap("B", 1.0, 0.0, 1.0)]).validate(),
            Err(LayoutError::Overlap { .. })
        ));
        assert!(matches!(
            layout(vec![cap("A", -1.0, 0.0, 1.0)]).validate(),
            Err(LayoutError::BadRect(_))
        ));
        assert!(matches!(
            layout(vec![cap("A", 0.0, 0.0, f32::NAN)]).validate(),
            Err(LayoutError::BadRect(_))
        ));
        let mut long = cap("A", 0.0, 0.0, 1.0);
        long.label = "x".repeat(MAX_LABEL_LEN + 1);
        assert!(matches!(
            layout(vec![long]).validate(),
            Err(LayoutError::LongLabel(_))
        ));
    }

    #[test]
    fn bounds_and_hit_test() {
        let l = layout(vec![cap("A", 0.0, 0.0, 1.0), cap("B", 2.0, 1.0, 2.0)]);
        assert_eq!(l.bounds(), Some(Rect::new(0.0, 0.0, 4.0, 2.0)));
        assert_eq!(l.key_at(2.5, 1.5).map(|k| k.code.as_str()), Some("B"));
        assert!(l.key_at(1.5, 0.5).is_none());
    }

    proptest! {
        /// Arbitrary (deserialised, untrusted) layouts: validate never panics.
        #[test]
        fn validate_never_panics(
            keys in proptest::collection::vec(
                ("[A-Z]{1,3}", any::<f32>(), any::<f32>(), any::<f32>(), any::<f32>(), ".{0,20}"),
                0..40,
            )
        ) {
            let keys = keys
                .into_iter()
                .map(|(c, x, y, w, h, label)| KeyCap {
                    code: KeyCode::new(&c).unwrap(),
                    rect: Rect::new(x, y, w, h),
                    label,
                    shift_label: String::new(),
                })
                .collect();
            let l = layout(keys);
            let _ = l.validate();
            let _ = l.bounds();
        }
    }
}
