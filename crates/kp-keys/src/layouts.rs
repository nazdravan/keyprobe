//! Built-in layouts. Geometry follows the common full-size ANSI board (standard key widths in u:
//! Backspace 2, Tab 1.5, Caps Lock 1.75, Enter 2.25, left Shift 2.25, right Shift 2.75, bottom-row
//! modifiers 1.25, Space 6.25; keypad `+`/Enter 2u tall, keypad 0 2u wide). Legends are US English.

use kp_doc::{KeyCap, KeyCode, Layout};
use kp_geom::Rect;

/// Ids accepted by `view.layout`, in menu order.
pub const LAYOUT_NAMES: &[&str] = &["ansi-104"];

/// `(code, x, y, w, h, label, shift_label)` in key units.
type Row = (&'static str, f32, f32, f32, f32, &'static str, &'static str);

/// Function row at y = 0; the main block starts after a 0.5u gap.
const R0: f32 = 0.0;
const R1: f32 = 1.5;
const R2: f32 = 2.5;
const R3: f32 = 3.5;
const R4: f32 = 4.5;
const R5: f32 = 5.5;
/// Left edge of the navigation cluster and of the keypad.
const NAV: f32 = 15.25;
const PAD: f32 = 18.5;

#[rustfmt::skip]
static ANSI_104: &[Row] = &[
    // Function row.
    ("Escape", 0.0, R0, 1.0, 1.0, "Esc", ""),
    ("F1", 2.0, R0, 1.0, 1.0, "F1", ""),
    ("F2", 3.0, R0, 1.0, 1.0, "F2", ""),
    ("F3", 4.0, R0, 1.0, 1.0, "F3", ""),
    ("F4", 5.0, R0, 1.0, 1.0, "F4", ""),
    ("F5", 6.5, R0, 1.0, 1.0, "F5", ""),
    ("F6", 7.5, R0, 1.0, 1.0, "F6", ""),
    ("F7", 8.5, R0, 1.0, 1.0, "F7", ""),
    ("F8", 9.5, R0, 1.0, 1.0, "F8", ""),
    ("F9", 11.0, R0, 1.0, 1.0, "F9", ""),
    ("F10", 12.0, R0, 1.0, 1.0, "F10", ""),
    ("F11", 13.0, R0, 1.0, 1.0, "F11", ""),
    ("F12", 14.0, R0, 1.0, 1.0, "F12", ""),
    ("PrintScreen", NAV, R0, 1.0, 1.0, "PrtSc", ""),
    ("ScrollLock", NAV + 1.0, R0, 1.0, 1.0, "ScrLk", ""),
    ("Pause", NAV + 2.0, R0, 1.0, 1.0, "Pause", "Break"),
    // Number row.
    ("Backquote", 0.0, R1, 1.0, 1.0, "`", "~"),
    ("Digit1", 1.0, R1, 1.0, 1.0, "1", "!"),
    ("Digit2", 2.0, R1, 1.0, 1.0, "2", "@"),
    ("Digit3", 3.0, R1, 1.0, 1.0, "3", "#"),
    ("Digit4", 4.0, R1, 1.0, 1.0, "4", "$"),
    ("Digit5", 5.0, R1, 1.0, 1.0, "5", "%"),
    ("Digit6", 6.0, R1, 1.0, 1.0, "6", "^"),
    ("Digit7", 7.0, R1, 1.0, 1.0, "7", "&"),
    ("Digit8", 8.0, R1, 1.0, 1.0, "8", "*"),
    ("Digit9", 9.0, R1, 1.0, 1.0, "9", "("),
    ("Digit0", 10.0, R1, 1.0, 1.0, "0", ")"),
    ("Minus", 11.0, R1, 1.0, 1.0, "-", "_"),
    ("Equal", 12.0, R1, 1.0, 1.0, "=", "+"),
    ("Backspace", 13.0, R1, 2.0, 1.0, "Backspace", ""),
    ("Insert", NAV, R1, 1.0, 1.0, "Ins", ""),
    ("Home", NAV + 1.0, R1, 1.0, 1.0, "Home", ""),
    ("PageUp", NAV + 2.0, R1, 1.0, 1.0, "PgUp", ""),
    ("NumLock", PAD, R1, 1.0, 1.0, "Num", ""),
    ("NumpadDivide", PAD + 1.0, R1, 1.0, 1.0, "/", ""),
    ("NumpadMultiply", PAD + 2.0, R1, 1.0, 1.0, "*", ""),
    ("NumpadSubtract", PAD + 3.0, R1, 1.0, 1.0, "-", ""),
    // Top letter row.
    ("Tab", 0.0, R2, 1.5, 1.0, "Tab", ""),
    ("KeyQ", 1.5, R2, 1.0, 1.0, "Q", ""),
    ("KeyW", 2.5, R2, 1.0, 1.0, "W", ""),
    ("KeyE", 3.5, R2, 1.0, 1.0, "E", ""),
    ("KeyR", 4.5, R2, 1.0, 1.0, "R", ""),
    ("KeyT", 5.5, R2, 1.0, 1.0, "T", ""),
    ("KeyY", 6.5, R2, 1.0, 1.0, "Y", ""),
    ("KeyU", 7.5, R2, 1.0, 1.0, "U", ""),
    ("KeyI", 8.5, R2, 1.0, 1.0, "I", ""),
    ("KeyO", 9.5, R2, 1.0, 1.0, "O", ""),
    ("KeyP", 10.5, R2, 1.0, 1.0, "P", ""),
    ("BracketLeft", 11.5, R2, 1.0, 1.0, "[", "{"),
    ("BracketRight", 12.5, R2, 1.0, 1.0, "]", "}"),
    ("Backslash", 13.5, R2, 1.5, 1.0, "\\", "|"),
    ("Delete", NAV, R2, 1.0, 1.0, "Del", ""),
    ("End", NAV + 1.0, R2, 1.0, 1.0, "End", ""),
    ("PageDown", NAV + 2.0, R2, 1.0, 1.0, "PgDn", ""),
    ("Numpad7", PAD, R2, 1.0, 1.0, "7", "Home"),
    ("Numpad8", PAD + 1.0, R2, 1.0, 1.0, "8", "↑"),
    ("Numpad9", PAD + 2.0, R2, 1.0, 1.0, "9", "PgUp"),
    ("NumpadAdd", PAD + 3.0, R2, 1.0, 2.0, "+", ""),
    // Home row.
    ("CapsLock", 0.0, R3, 1.75, 1.0, "Caps Lock", ""),
    ("KeyA", 1.75, R3, 1.0, 1.0, "A", ""),
    ("KeyS", 2.75, R3, 1.0, 1.0, "S", ""),
    ("KeyD", 3.75, R3, 1.0, 1.0, "D", ""),
    ("KeyF", 4.75, R3, 1.0, 1.0, "F", ""),
    ("KeyG", 5.75, R3, 1.0, 1.0, "G", ""),
    ("KeyH", 6.75, R3, 1.0, 1.0, "H", ""),
    ("KeyJ", 7.75, R3, 1.0, 1.0, "J", ""),
    ("KeyK", 8.75, R3, 1.0, 1.0, "K", ""),
    ("KeyL", 9.75, R3, 1.0, 1.0, "L", ""),
    ("Semicolon", 10.75, R3, 1.0, 1.0, ";", ":"),
    ("Quote", 11.75, R3, 1.0, 1.0, "'", "\""),
    ("Enter", 12.75, R3, 2.25, 1.0, "Enter", ""),
    ("Numpad4", PAD, R3, 1.0, 1.0, "4", "←"),
    ("Numpad5", PAD + 1.0, R3, 1.0, 1.0, "5", ""),
    ("Numpad6", PAD + 2.0, R3, 1.0, 1.0, "6", "→"),
    // Bottom letter row.
    ("ShiftLeft", 0.0, R4, 2.25, 1.0, "Shift", ""),
    ("KeyZ", 2.25, R4, 1.0, 1.0, "Z", ""),
    ("KeyX", 3.25, R4, 1.0, 1.0, "X", ""),
    ("KeyC", 4.25, R4, 1.0, 1.0, "C", ""),
    ("KeyV", 5.25, R4, 1.0, 1.0, "V", ""),
    ("KeyB", 6.25, R4, 1.0, 1.0, "B", ""),
    ("KeyN", 7.25, R4, 1.0, 1.0, "N", ""),
    ("KeyM", 8.25, R4, 1.0, 1.0, "M", ""),
    ("Comma", 9.25, R4, 1.0, 1.0, ",", "<"),
    ("Period", 10.25, R4, 1.0, 1.0, ".", ">"),
    ("Slash", 11.25, R4, 1.0, 1.0, "/", "?"),
    ("ShiftRight", 12.25, R4, 2.75, 1.0, "Shift", ""),
    ("ArrowUp", NAV + 1.0, R4, 1.0, 1.0, "↑", ""),
    ("Numpad1", PAD, R4, 1.0, 1.0, "1", "End"),
    ("Numpad2", PAD + 1.0, R4, 1.0, 1.0, "2", "↓"),
    ("Numpad3", PAD + 2.0, R4, 1.0, 1.0, "3", "PgDn"),
    ("NumpadEnter", PAD + 3.0, R4, 1.0, 2.0, "Enter", ""),
    // Space row.
    ("ControlLeft", 0.0, R5, 1.25, 1.0, "Ctrl", ""),
    ("MetaLeft", 1.25, R5, 1.25, 1.0, "Win", ""),
    ("AltLeft", 2.5, R5, 1.25, 1.0, "Alt", ""),
    ("Space", 3.75, R5, 6.25, 1.0, "Space", ""),
    ("AltRight", 10.0, R5, 1.25, 1.0, "Alt", ""),
    ("MetaRight", 11.25, R5, 1.25, 1.0, "Win", ""),
    ("ContextMenu", 12.5, R5, 1.25, 1.0, "Menu", ""),
    ("ControlRight", 13.75, R5, 1.25, 1.0, "Ctrl", ""),
    ("ArrowLeft", NAV, R5, 1.0, 1.0, "←", ""),
    ("ArrowDown", NAV + 1.0, R5, 1.0, 1.0, "↓", ""),
    ("ArrowRight", NAV + 2.0, R5, 1.0, 1.0, "→", ""),
    ("Numpad0", PAD, R5, 2.0, 1.0, "0", "Ins"),
    ("NumpadDecimal", PAD + 2.0, R5, 1.0, 1.0, ".", "Del"),
];

/// A built-in layout by id.
pub fn layout(name: &str) -> Option<Layout> {
    match name {
        "ansi-104" => Some(build("ansi-104", "ANSI 104 (full size)", ANSI_104)),
        _ => None,
    }
}

fn build(name: &str, title: &str, rows: &[Row]) -> Layout {
    Layout {
        name: name.to_owned(),
        title: title.to_owned(),
        // Codes in the static tables are valid W3C names (checked by tests), so nothing is dropped.
        keys: rows
            .iter()
            .filter_map(|&(code, x, y, w, h, label, shift)| {
                Some(KeyCap {
                    code: KeyCode::new(code).ok()?,
                    rect: Rect::new(x, y, w, h),
                    label: label.to_owned(),
                    shift_label: shift.to_owned(),
                })
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::by_code;

    #[test]
    fn every_builtin_layout_is_valid_and_known() {
        for name in LAYOUT_NAMES {
            let l = layout(name).unwrap();
            assert_eq!(&l.name, name);
            l.validate().unwrap();
            for key in &l.keys {
                assert!(
                    by_code(key.code.as_str()).is_some(),
                    "{name}: {} not in key table",
                    key.code
                );
            }
        }
        assert!(layout("nope").is_none());
    }

    #[test]
    fn ansi_104_shape() {
        let l = layout("ansi-104").unwrap();
        assert_eq!(l.keys.len(), ANSI_104.len());
        assert_eq!(l.keys.len(), 104);
        assert_eq!(l.bounds(), Some(Rect::new(0.0, 0.0, 22.5, 6.5)));
        // Each row of the main block is exactly 15u wide.
        for y in [R1, R2, R3, R4, R5] {
            let right = l
                .keys
                .iter()
                .filter(|k| k.rect.y == y && k.rect.x < NAV)
                .map(|k| k.rect.right())
                .fold(0.0, f32::max);
            let width: f32 = l
                .keys
                .iter()
                .filter(|k| k.rect.y == y && k.rect.x < NAV)
                .map(|k| k.rect.w)
                .sum();
            assert_eq!(right, 15.0, "row y={y} ends at {right}");
            assert_eq!(width, 15.0, "row y={y} has gaps");
        }
        let code = |c: &str| KeyCode::new(c).unwrap();
        assert_eq!(l.key(&code("Space")).unwrap().rect.w, 6.25);
        assert_eq!(l.key(&code("NumpadEnter")).unwrap().rect.h, 2.0);
        assert_eq!(l.key_at(19.0, 6.0).unwrap().code.as_str(), "Numpad0");
    }
}
