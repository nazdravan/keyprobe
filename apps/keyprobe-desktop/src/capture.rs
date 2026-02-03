//! Raw key capture: winit `KeyboardInput` → `key.down` / `key.up` command params.
//!
//! We read winit's physical key (layout-independent, named after the W3C `code` values) and the
//! platform scancode, which egui's own `Key` drops.

use std::sync::OnceLock;
use std::time::Instant;

use serde_json::{Value, json};
use winit::keyboard::{Key, PhysicalKey};
use winit::platform::scancode::PhysicalKeyExtScancode;

/// Something the window saw that becomes a command on the UI thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Captured {
    Key(RawKey),
    /// The window lost focus: key-ups for held keys will go elsewhere, so drop them.
    ReleaseAll,
}

impl Captured {
    pub fn command(&self) -> (&'static str, Value) {
        match self {
            Captured::Key(raw) => raw.command(),
            Captured::ReleaseAll => ("key.release_all", Value::Null),
        }
    }
}

/// One captured key event, ready to become a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawKey {
    pub code: String,
    pub down: bool,
    pub repeat: bool,
    pub scancode: Option<u32>,
    pub key: Option<String>,
    /// Capture time in ms since the first captured event's clock start (see [`now_ms`]).
    pub time_ms: Option<u64>,
}

impl RawKey {
    /// `(command id, params)` for the engine.
    pub fn command(&self) -> (&'static str, Value) {
        let mut native = serde_json::Map::new();
        if let Some(sc) = self.scancode {
            native.insert("scancode".into(), json!(sc));
        }
        if let Some(key) = &self.key {
            native.insert("key".into(), json!(key));
        }
        let mut params = json!({"code": self.code, "native": native});
        if let (Some(t), Some(obj)) = (self.time_ms, params.as_object_mut()) {
            obj.insert("time_ms".into(), json!(t));
        }
        if self.down {
            if let Some(obj) = params.as_object_mut() {
                obj.insert("repeat".into(), json!(self.repeat));
            }
            ("key.down", params)
        } else {
            ("key.up", params)
        }
    }
}

/// Builds a [`RawKey`]; `None` for keys winit can't identify (no physical code to test).
pub fn raw_key(physical: PhysicalKey, logical: &Key, down: bool, repeat: bool) -> Option<RawKey> {
    Some(RawKey {
        code: w3c_code(physical)?,
        down,
        repeat,
        scancode: physical.to_scancode(),
        key: key_text(logical),
        time_ms: Some(now_ms()),
    })
}

/// Milliseconds on a monotonic clock that starts at the first call.
pub fn now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    let elapsed = START.get_or_init(Instant::now).elapsed().as_millis();
    u64::try_from(elapsed).unwrap_or(u64::MAX)
}

/// winit `KeyCode` variant names follow the W3C `code` values; the few that differ
/// (`SuperLeft` for `MetaLeft`…) are resolved by `kp_keys::canonical`.
pub fn w3c_code(physical: PhysicalKey) -> Option<String> {
    match physical {
        PhysicalKey::Code(code) => Some(kp_keys::canonical(&format!("{code:?}")).to_owned()),
        PhysicalKey::Unidentified(_) => None,
    }
}

fn key_text(logical: &Key) -> Option<String> {
    match logical {
        Key::Character(s) => Some(s.to_string()),
        Key::Named(named) => Some(format!("{named:?}")),
        Key::Dead(c) => Some(c.map_or_else(|| "Dead".to_owned(), |c| format!("Dead {c}"))),
        Key::Unidentified(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use winit::keyboard::{KeyCode as K, NamedKey, NativeKeyCode};

    /// The winit codes a full-size ANSI board produces, plus F13–F24 and the ISO key.
    const BOARD: &[K] = &[
        K::KeyA,
        K::KeyB,
        K::KeyC,
        K::KeyD,
        K::KeyE,
        K::KeyF,
        K::KeyG,
        K::KeyH,
        K::KeyI,
        K::KeyJ,
        K::KeyK,
        K::KeyL,
        K::KeyM,
        K::KeyN,
        K::KeyO,
        K::KeyP,
        K::KeyQ,
        K::KeyR,
        K::KeyS,
        K::KeyT,
        K::KeyU,
        K::KeyV,
        K::KeyW,
        K::KeyX,
        K::KeyY,
        K::KeyZ,
        K::Digit0,
        K::Digit1,
        K::Digit2,
        K::Digit3,
        K::Digit4,
        K::Digit5,
        K::Digit6,
        K::Digit7,
        K::Digit8,
        K::Digit9,
        K::Numpad0,
        K::Numpad1,
        K::Numpad2,
        K::Numpad3,
        K::Numpad4,
        K::Numpad5,
        K::Numpad6,
        K::Numpad7,
        K::Numpad8,
        K::Numpad9,
        K::F1,
        K::F2,
        K::F3,
        K::F4,
        K::F5,
        K::F6,
        K::F7,
        K::F8,
        K::F9,
        K::F10,
        K::F11,
        K::F12,
        K::F13,
        K::F14,
        K::F15,
        K::F16,
        K::F17,
        K::F18,
        K::F19,
        K::F20,
        K::F21,
        K::F22,
        K::F23,
        K::F24,
        K::Escape,
        K::Backquote,
        K::Minus,
        K::Equal,
        K::Backspace,
        K::Tab,
        K::BracketLeft,
        K::BracketRight,
        K::Backslash,
        K::CapsLock,
        K::Semicolon,
        K::Quote,
        K::Enter,
        K::ShiftLeft,
        K::Comma,
        K::Period,
        K::Slash,
        K::ShiftRight,
        K::ControlLeft,
        K::SuperLeft,
        K::AltLeft,
        K::Space,
        K::AltRight,
        K::SuperRight,
        K::ContextMenu,
        K::ControlRight,
        K::PrintScreen,
        K::ScrollLock,
        K::Pause,
        K::Insert,
        K::Home,
        K::PageUp,
        K::Delete,
        K::End,
        K::PageDown,
        K::ArrowUp,
        K::ArrowLeft,
        K::ArrowDown,
        K::ArrowRight,
        K::NumLock,
        K::NumpadDivide,
        K::NumpadMultiply,
        K::NumpadSubtract,
        K::NumpadAdd,
        K::NumpadEnter,
        K::NumpadDecimal,
        K::IntlBackslash,
    ];

    /// Non-US, media and browser keys (G15).
    const EXTRA: &[K] = &[
        K::NumpadEqual,
        K::IntlRo,
        K::KanaMode,
        K::IntlYen,
        K::Convert,
        K::NonConvert,
        K::MediaTrackPrevious,
        K::MediaTrackNext,
        K::AudioVolumeMute,
        K::MediaPlayPause,
        K::MediaStop,
        K::AudioVolumeDown,
        K::AudioVolumeUp,
        K::BrowserHome,
        K::BrowserSearch,
        K::BrowserFavorites,
        K::BrowserRefresh,
        K::BrowserStop,
        K::BrowserForward,
        K::BrowserBack,
        K::LaunchMail,
    ];

    #[test]
    fn every_board_key_maps_into_the_key_table() {
        let mut seen = BTreeSet::new();
        for &k in BOARD.iter().chain(EXTRA) {
            let code = w3c_code(PhysicalKey::Code(k)).unwrap();
            assert!(
                kp_keys::by_code(&code).is_some(),
                "{k:?} → {code} not in key table"
            );
            assert!(seen.insert(code.clone()), "{k:?} → {code} twice");
        }
        assert_eq!(seen.len(), kp_keys::KEYS.len());
        let ansi = kp_keys::layout("ansi-104").unwrap();
        for key in &ansi.keys {
            assert!(
                seen.contains(key.code.as_str()),
                "{} unreachable from winit",
                key.code
            );
        }
    }

    #[test]
    fn unidentified_keys_are_skipped() {
        let k = PhysicalKey::Unidentified(NativeKeyCode::Unidentified);
        assert!(
            raw_key(
                k,
                &Key::Unidentified(winit::keyboard::NativeKey::Unidentified),
                true,
                false
            )
            .is_none()
        );
    }

    #[test]
    fn commands_carry_code_repeat_and_native() {
        let raw = raw_key(
            PhysicalKey::Code(K::SuperLeft),
            &Key::Named(NamedKey::Super),
            true,
            true,
        )
        .unwrap();
        let (id, params) = raw.command();
        assert_eq!(id, "key.down");
        assert_eq!(params["code"], "MetaLeft");
        assert_eq!(params["repeat"], true);
        assert_eq!(params["native"]["key"], "Super");

        let up = RawKey { down: false, ..raw };
        let (id, params) = up.command();
        assert_eq!(id, "key.up");
        assert!(params.get("repeat").is_none());
    }

    #[test]
    fn focus_loss_releases_held_keys() {
        let mut app = kp_engine::App::default();
        let down = raw_key(
            PhysicalKey::Code(K::KeyA),
            &Key::Character("a".into()),
            true,
            false,
        )
        .unwrap();
        for c in [Captured::Key(down), Captured::ReleaseAll] {
            let (id, params) = c.command();
            app.run(id, &params).unwrap();
        }
        assert!(app.session().doc().test.held.is_empty());
        assert_eq!(app.session().doc().test.presses.len(), 1);
    }

    #[test]
    fn captured_params_are_accepted_by_the_engine() {
        let mut app = kp_engine::App::default();
        for &k in BOARD {
            for down in [true, false] {
                let raw = raw_key(
                    PhysicalKey::Code(k),
                    &Key::Character("x".into()),
                    down,
                    false,
                )
                .unwrap();
                let (id, params) = raw.command();
                app.run(id, &params).unwrap();
            }
        }
        assert_eq!(app.session().doc().test.presses.len(), BOARD.len());
        assert!(app.session().doc().test.held.is_empty());
    }
}
