//! L2: physical key table (W3C code, USB HID, Windows VK, PS/2 Set 1) and built-in layouts.
//!
//! One row per physical key, keyed by its W3C UI Events `code`. The other columns let every
//! platform show the Windows-style readout (VK code + scan code) from the physical key alone.
//!
//! Sources (public specifications only, see docs/spec.md):
//! - W3C UI Events KeyboardEvent code Values — <https://www.w3.org/TR/uievents-code/>
//! - USB HID Usage Tables 1.5, Keyboard/Keypad page 0x07 — <https://usb.org/document-library/hid-usage-tables-15>
//! - Microsoft Virtual-Key Codes — <https://learn.microsoft.com/windows/win32/inputdev/virtual-key-codes>
//! - Microsoft Keyboard Scan Code Specification, Set 1 —
//!   <https://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/scancode.doc>
//! - Microsoft "About Keyboard Input", keystroke message flags (extended-key flag) —
//!   <https://learn.microsoft.com/windows/win32/inputdev/about-keyboard-input>
//!
//! Column meanings:
//! - `vk`: the `wParam` of `WM_KEYDOWN` with a US layout. Left and right modifiers share the
//!   generic code (`VK_SHIFT`, `VK_CONTROL`, `VK_MENU`), as in the message; the side is in `code`.
//!   Letters and digits have no `VK_` constant in the docs; we name them `VK_A`, `VK_0`.
//! - `scan` + `extended`: bits 16–23 and bit 24 of the message `lParam`, i.e. the Set 1 make code
//!   without its `E0` prefix plus the extended flag. Pause (Set 1 `E1 1D 45`) arrives as `0x45`
//!   without the flag; Num Lock (`45`) arrives as `0x45` *with* it, as the docs list Num Lock among
//!   the extended keys.

use kp_doc::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInfo {
    /// W3C `code` value.
    pub code: &'static str,
    /// USB HID usage id on the Keyboard/Keypad page (0x07).
    pub hid: u16,
    /// Windows virtual-key code.
    pub vk: u8,
    pub vk_name: &'static str,
    /// Windows scan code (lParam bits 16–23).
    pub scan: u8,
    /// Windows extended-key flag (lParam bit 24; `E0`-prefixed in Set 1).
    pub extended: bool,
}

const fn k(
    code: &'static str,
    hid: u16,
    vk: u8,
    vk_name: &'static str,
    scan: u8,
    extended: bool,
) -> KeyInfo {
    KeyInfo {
        code,
        hid,
        vk,
        vk_name,
        scan,
        extended,
    }
}

const N: bool = false;
const E: bool = true;

/// Every key we know, in HID usage order.
pub static KEYS: &[KeyInfo] = &[
    // Letters.
    k("KeyA", 0x04, 0x41, "VK_A", 0x1E, N),
    k("KeyB", 0x05, 0x42, "VK_B", 0x30, N),
    k("KeyC", 0x06, 0x43, "VK_C", 0x2E, N),
    k("KeyD", 0x07, 0x44, "VK_D", 0x20, N),
    k("KeyE", 0x08, 0x45, "VK_E", 0x12, N),
    k("KeyF", 0x09, 0x46, "VK_F", 0x21, N),
    k("KeyG", 0x0A, 0x47, "VK_G", 0x22, N),
    k("KeyH", 0x0B, 0x48, "VK_H", 0x23, N),
    k("KeyI", 0x0C, 0x49, "VK_I", 0x17, N),
    k("KeyJ", 0x0D, 0x4A, "VK_J", 0x24, N),
    k("KeyK", 0x0E, 0x4B, "VK_K", 0x25, N),
    k("KeyL", 0x0F, 0x4C, "VK_L", 0x26, N),
    k("KeyM", 0x10, 0x4D, "VK_M", 0x32, N),
    k("KeyN", 0x11, 0x4E, "VK_N", 0x31, N),
    k("KeyO", 0x12, 0x4F, "VK_O", 0x18, N),
    k("KeyP", 0x13, 0x50, "VK_P", 0x19, N),
    k("KeyQ", 0x14, 0x51, "VK_Q", 0x10, N),
    k("KeyR", 0x15, 0x52, "VK_R", 0x13, N),
    k("KeyS", 0x16, 0x53, "VK_S", 0x1F, N),
    k("KeyT", 0x17, 0x54, "VK_T", 0x14, N),
    k("KeyU", 0x18, 0x55, "VK_U", 0x16, N),
    k("KeyV", 0x19, 0x56, "VK_V", 0x2F, N),
    k("KeyW", 0x1A, 0x57, "VK_W", 0x11, N),
    k("KeyX", 0x1B, 0x58, "VK_X", 0x2D, N),
    k("KeyY", 0x1C, 0x59, "VK_Y", 0x15, N),
    k("KeyZ", 0x1D, 0x5A, "VK_Z", 0x2C, N),
    // Digit row.
    k("Digit1", 0x1E, 0x31, "VK_1", 0x02, N),
    k("Digit2", 0x1F, 0x32, "VK_2", 0x03, N),
    k("Digit3", 0x20, 0x33, "VK_3", 0x04, N),
    k("Digit4", 0x21, 0x34, "VK_4", 0x05, N),
    k("Digit5", 0x22, 0x35, "VK_5", 0x06, N),
    k("Digit6", 0x23, 0x36, "VK_6", 0x07, N),
    k("Digit7", 0x24, 0x37, "VK_7", 0x08, N),
    k("Digit8", 0x25, 0x38, "VK_8", 0x09, N),
    k("Digit9", 0x26, 0x39, "VK_9", 0x0A, N),
    k("Digit0", 0x27, 0x30, "VK_0", 0x0B, N),
    // Control and punctuation.
    k("Enter", 0x28, 0x0D, "VK_RETURN", 0x1C, N),
    k("Escape", 0x29, 0x1B, "VK_ESCAPE", 0x01, N),
    k("Backspace", 0x2A, 0x08, "VK_BACK", 0x0E, N),
    k("Tab", 0x2B, 0x09, "VK_TAB", 0x0F, N),
    k("Space", 0x2C, 0x20, "VK_SPACE", 0x39, N),
    k("Minus", 0x2D, 0xBD, "VK_OEM_MINUS", 0x0C, N),
    k("Equal", 0x2E, 0xBB, "VK_OEM_PLUS", 0x0D, N),
    k("BracketLeft", 0x2F, 0xDB, "VK_OEM_4", 0x1A, N),
    k("BracketRight", 0x30, 0xDD, "VK_OEM_6", 0x1B, N),
    k("Backslash", 0x31, 0xDC, "VK_OEM_5", 0x2B, N),
    k("Semicolon", 0x33, 0xBA, "VK_OEM_1", 0x27, N),
    k("Quote", 0x34, 0xDE, "VK_OEM_7", 0x28, N),
    k("Backquote", 0x35, 0xC0, "VK_OEM_3", 0x29, N),
    k("Comma", 0x36, 0xBC, "VK_OEM_COMMA", 0x33, N),
    k("Period", 0x37, 0xBE, "VK_OEM_PERIOD", 0x34, N),
    k("Slash", 0x38, 0xBF, "VK_OEM_2", 0x35, N),
    k("CapsLock", 0x39, 0x14, "VK_CAPITAL", 0x3A, N),
    // F1–F12.
    k("F1", 0x3A, 0x70, "VK_F1", 0x3B, N),
    k("F2", 0x3B, 0x71, "VK_F2", 0x3C, N),
    k("F3", 0x3C, 0x72, "VK_F3", 0x3D, N),
    k("F4", 0x3D, 0x73, "VK_F4", 0x3E, N),
    k("F5", 0x3E, 0x74, "VK_F5", 0x3F, N),
    k("F6", 0x3F, 0x75, "VK_F6", 0x40, N),
    k("F7", 0x40, 0x76, "VK_F7", 0x41, N),
    k("F8", 0x41, 0x77, "VK_F8", 0x42, N),
    k("F9", 0x42, 0x78, "VK_F9", 0x43, N),
    k("F10", 0x43, 0x79, "VK_F10", 0x44, N),
    k("F11", 0x44, 0x7A, "VK_F11", 0x57, N),
    k("F12", 0x45, 0x7B, "VK_F12", 0x58, N),
    // System and navigation cluster.
    k("PrintScreen", 0x46, 0x2C, "VK_SNAPSHOT", 0x37, E),
    k("ScrollLock", 0x47, 0x91, "VK_SCROLL", 0x46, N),
    k("Pause", 0x48, 0x13, "VK_PAUSE", 0x45, N),
    k("Insert", 0x49, 0x2D, "VK_INSERT", 0x52, E),
    k("Home", 0x4A, 0x24, "VK_HOME", 0x47, E),
    k("PageUp", 0x4B, 0x21, "VK_PRIOR", 0x49, E),
    k("Delete", 0x4C, 0x2E, "VK_DELETE", 0x53, E),
    k("End", 0x4D, 0x23, "VK_END", 0x4F, E),
    k("PageDown", 0x4E, 0x22, "VK_NEXT", 0x51, E),
    k("ArrowRight", 0x4F, 0x27, "VK_RIGHT", 0x4D, E),
    k("ArrowLeft", 0x50, 0x25, "VK_LEFT", 0x4B, E),
    k("ArrowDown", 0x51, 0x28, "VK_DOWN", 0x50, E),
    k("ArrowUp", 0x52, 0x26, "VK_UP", 0x48, E),
    // Numeric keypad.
    k("NumLock", 0x53, 0x90, "VK_NUMLOCK", 0x45, E),
    k("NumpadDivide", 0x54, 0x6F, "VK_DIVIDE", 0x35, E),
    k("NumpadMultiply", 0x55, 0x6A, "VK_MULTIPLY", 0x37, N),
    k("NumpadSubtract", 0x56, 0x6D, "VK_SUBTRACT", 0x4A, N),
    k("NumpadAdd", 0x57, 0x6B, "VK_ADD", 0x4E, N),
    k("NumpadEnter", 0x58, 0x0D, "VK_RETURN", 0x1C, E),
    k("Numpad1", 0x59, 0x61, "VK_NUMPAD1", 0x4F, N),
    k("Numpad2", 0x5A, 0x62, "VK_NUMPAD2", 0x50, N),
    k("Numpad3", 0x5B, 0x63, "VK_NUMPAD3", 0x51, N),
    k("Numpad4", 0x5C, 0x64, "VK_NUMPAD4", 0x4B, N),
    k("Numpad5", 0x5D, 0x65, "VK_NUMPAD5", 0x4C, N),
    k("Numpad6", 0x5E, 0x66, "VK_NUMPAD6", 0x4D, N),
    k("Numpad7", 0x5F, 0x67, "VK_NUMPAD7", 0x47, N),
    k("Numpad8", 0x60, 0x68, "VK_NUMPAD8", 0x48, N),
    k("Numpad9", 0x61, 0x69, "VK_NUMPAD9", 0x49, N),
    k("Numpad0", 0x62, 0x60, "VK_NUMPAD0", 0x52, N),
    k("NumpadDecimal", 0x63, 0x6E, "VK_DECIMAL", 0x53, N),
    // ISO key next to left Shift, and the application (menu) key.
    k("IntlBackslash", 0x64, 0xE2, "VK_OEM_102", 0x56, N),
    k("ContextMenu", 0x65, 0x5D, "VK_APPS", 0x5D, E),
    // F13–F24.
    k("F13", 0x68, 0x7C, "VK_F13", 0x64, N),
    k("F14", 0x69, 0x7D, "VK_F14", 0x65, N),
    k("F15", 0x6A, 0x7E, "VK_F15", 0x66, N),
    k("F16", 0x6B, 0x7F, "VK_F16", 0x67, N),
    k("F17", 0x6C, 0x80, "VK_F17", 0x68, N),
    k("F18", 0x6D, 0x81, "VK_F18", 0x69, N),
    k("F19", 0x6E, 0x82, "VK_F19", 0x6A, N),
    k("F20", 0x6F, 0x83, "VK_F20", 0x6B, N),
    k("F21", 0x70, 0x84, "VK_F21", 0x6C, N),
    k("F22", 0x71, 0x85, "VK_F22", 0x6D, N),
    k("F23", 0x72, 0x86, "VK_F23", 0x6E, N),
    k("F24", 0x73, 0x87, "VK_F24", 0x76, N),
    // Modifiers.
    k("ControlLeft", 0xE0, 0x11, "VK_CONTROL", 0x1D, N),
    k("ShiftLeft", 0xE1, 0x10, "VK_SHIFT", 0x2A, N),
    k("AltLeft", 0xE2, 0x12, "VK_MENU", 0x38, N),
    k("MetaLeft", 0xE3, 0x5B, "VK_LWIN", 0x5B, E),
    k("ControlRight", 0xE4, 0x11, "VK_CONTROL", 0x1D, E),
    k("ShiftRight", 0xE5, 0x10, "VK_SHIFT", 0x36, N),
    k("AltRight", 0xE6, 0x12, "VK_MENU", 0x38, E),
    k("MetaRight", 0xE7, 0x5C, "VK_RWIN", 0x5C, E),
];

/// Other names platforms use for the same physical key, mapped to the W3C name.
/// winit names the Windows/Command keys `Super*`; older Firefox sent `OS*`.
static ALIASES: &[(&str, &str)] = &[
    ("SuperLeft", "MetaLeft"),
    ("SuperRight", "MetaRight"),
    ("OSLeft", "MetaLeft"),
    ("OSRight", "MetaRight"),
];

/// The W3C name for `code`, resolving known aliases. Unknown codes come back unchanged.
pub fn canonical(code: &str) -> &str {
    ALIASES
        .iter()
        .find(|(alias, _)| *alias == code)
        .map_or(code, |(_, w3c)| w3c)
}

/// Looks a key up by its W3C `code` (or a known alias).
pub fn by_code(code: &str) -> Option<&'static KeyInfo> {
    let code = canonical(code);
    KEYS.iter().find(|k| k.code == code)
}

pub fn info(code: &KeyCode) -> Option<&'static KeyInfo> {
    by_code(code.as_str())
}

/// Looks a key up by its Windows scan code and extended flag (unique per key).
pub fn by_scan(scan: u8, extended: bool) -> Option<&'static KeyInfo> {
    KEYS.iter()
        .find(|k| k.scan == scan && k.extended == extended)
}

pub fn by_hid(usage: u16) -> Option<&'static KeyInfo> {
    KEYS.iter().find(|k| k.hid == usage)
}

/// `0x20 (32)`, the readout format of the reference app's status bar.
pub fn hex_dec(value: u8) -> String {
    format!("0x{value:02X} ({value})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    /// Values visible in the reference app's public screenshots.
    #[test]
    fn matches_reference_screenshots() {
        let space = by_code("Space").unwrap();
        assert_eq!((space.vk, space.vk_name), (0x20, "VK_SPACE"));
        assert_eq!(hex_dec(space.vk), "0x20 (32)");
        assert_eq!(hex_dec(space.scan), "0x39 (57)");
        assert!(!space.extended);

        let enter = by_code("Enter").unwrap();
        assert_eq!(
            (enter.vk, enter.vk_name, enter.scan),
            (0x0D, "VK_RETURN", 0x1C)
        );
        assert_eq!(hex_dec(enter.scan), "0x1C (28)");

        let prt = by_code("PrintScreen").unwrap();
        assert_eq!((prt.vk, prt.vk_name, prt.scan), (0x2C, "VK_SNAPSHOT", 0x37));
        assert_eq!(hex_dec(prt.vk), "0x2C (44)");
        assert!(prt.extended);
    }

    #[test]
    fn full_size_ansi_keys_are_all_present() {
        let mut want: Vec<String> = Vec::new();
        want.extend(('A'..='Z').map(|c| format!("Key{c}")));
        want.extend(('0'..='9').map(|c| format!("Digit{c}")));
        want.extend(('0'..='9').map(|c| format!("Numpad{c}")));
        want.extend((1..=12).map(|n| format!("F{n}")));
        for c in [
            "Escape",
            "Backquote",
            "Minus",
            "Equal",
            "Backspace",
            "Tab",
            "BracketLeft",
            "BracketRight",
            "Backslash",
            "CapsLock",
            "Semicolon",
            "Quote",
            "Enter",
            "ShiftLeft",
            "Comma",
            "Period",
            "Slash",
            "ShiftRight",
            "ControlLeft",
            "MetaLeft",
            "AltLeft",
            "Space",
            "AltRight",
            "MetaRight",
            "ContextMenu",
            "ControlRight",
            "PrintScreen",
            "ScrollLock",
            "Pause",
            "Insert",
            "Home",
            "PageUp",
            "Delete",
            "End",
            "PageDown",
            "ArrowUp",
            "ArrowLeft",
            "ArrowDown",
            "ArrowRight",
            "NumLock",
            "NumpadDivide",
            "NumpadMultiply",
            "NumpadSubtract",
            "NumpadAdd",
            "NumpadEnter",
            "NumpadDecimal",
        ] {
            want.push(c.to_owned());
        }
        assert_eq!(want.len(), 104);
        for code in &want {
            assert!(by_code(code).is_some(), "missing {code}");
        }
    }

    #[test]
    fn columns_are_unique_where_they_must_be() {
        let codes: BTreeSet<_> = KEYS.iter().map(|k| k.code).collect();
        let hids: BTreeSet<_> = KEYS.iter().map(|k| k.hid).collect();
        let scans: BTreeSet<_> = KEYS.iter().map(|k| (k.scan, k.extended)).collect();
        assert_eq!(codes.len(), KEYS.len(), "duplicate code");
        assert_eq!(hids.len(), KEYS.len(), "duplicate HID usage");
        assert_eq!(scans.len(), KEYS.len(), "duplicate scan code");
        for k in KEYS {
            assert!(
                KeyCode::new(k.code).is_ok(),
                "{} is not a valid KeyCode",
                k.code
            );
            assert!(k.vk_name.starts_with("VK_"), "{}", k.code);
            assert!(
                k.scan < 0x80,
                "{}: scan codes are make codes (< 0x80)",
                k.code
            );
        }
    }

    #[test]
    fn known_tricky_rows() {
        // Same scan code, told apart by the extended flag.
        assert_eq!(by_scan(0x45, false).unwrap().code, "Pause");
        assert_eq!(by_scan(0x45, true).unwrap().code, "NumLock");
        assert_eq!(by_scan(0x1C, true).unwrap().code, "NumpadEnter");
        assert_eq!(by_scan(0x47, false).unwrap().code, "Numpad7");
        assert_eq!(by_scan(0x47, true).unwrap().code, "Home");
        // Right Shift is not an extended key; right Ctrl/Alt are.
        assert!(!by_code("ShiftRight").unwrap().extended);
        assert!(by_code("ControlRight").unwrap().extended);
        assert!(by_code("AltRight").unwrap().extended);
        // Aliases.
        assert_eq!(by_code("SuperLeft").unwrap().code, "MetaLeft");
        assert_eq!(by_code("OSRight").unwrap().code, "MetaRight");
        assert_eq!(by_hid(0x2C).unwrap().code, "Space");
    }

    proptest! {
        #[test]
        fn lookups_never_panic(code in ".{0,40}", scan in any::<u8>(), ext in any::<bool>(), hid in any::<u16>()) {
            let _ = by_code(&code);
            let _ = canonical(&code);
            let _ = by_scan(scan, ext);
            let _ = by_hid(hid);
            let _ = hex_dec(scan);
        }

        #[test]
        fn scan_and_hid_round_trip(i in 0..KEYS.len()) {
            let key = &KEYS[i];
            prop_assert_eq!(by_scan(key.scan, key.extended), Some(key));
            prop_assert_eq!(by_hid(key.hid), Some(key));
            prop_assert_eq!(by_code(key.code), Some(key));
        }
    }
}
