//! L1: the test session document (layout, test state) as plain serde data.
//!
//! Everything here is untrusted-input safe: key codes are validated on construction and every
//! collection has a hard cap, so a flood of synthetic events can't grow memory without bound.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

/// Longest accepted W3C `code` name (the longest real one, `NumpadParenRight`, has 16 chars).
pub const MAX_CODE_LEN: usize = 32;
/// Most distinct keys tracked in one session (a full-size board has ~110; media/intl keys add some).
pub const MAX_KEYS: usize = 256;
/// Longest accepted `KeyboardEvent.key` text.
pub const MAX_KEY_TEXT_LEN: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DocError {
    #[error("invalid key code {0:?}: expected 1..={MAX_CODE_LEN} ASCII letters/digits")]
    InvalidCode(String),
    #[error("too many distinct keys (limit {MAX_KEYS})")]
    TooManyKeys,
}

/// Physical key id: a W3C UI Events `code` name such as `KeyA`, `ShiftLeft`, `NumpadEnter`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct KeyCode(String);

impl KeyCode {
    pub fn new(code: &str) -> Result<Self, DocError> {
        let ok = !code.is_empty()
            && code.len() <= MAX_CODE_LEN
            && code.bytes().all(|b| b.is_ascii_alphanumeric());
        if ok {
            Ok(Self(code.to_owned()))
        } else {
            Err(DocError::InvalidCode(
                code.chars().take(MAX_CODE_LEN).collect(),
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for KeyCode {
    type Error = DocError;
    fn try_from(s: String) -> Result<Self, DocError> {
        Self::new(&s)
    }
}

impl From<KeyCode> for String {
    fn from(c: KeyCode) -> String {
        c.0
    }
}

impl fmt::Display for KeyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Codes reported by the platform itself, next to the ones we derive from the key table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeCode {
    /// OS scancode (desktop, winit `PhysicalKeyExtScancode`).
    pub scancode: Option<u32>,
    /// Legacy `KeyboardEvent.keyCode` (web).
    pub key_code: Option<u32>,
    /// `KeyboardEvent.key` / logical key text, truncated to [`MAX_KEY_TEXT_LEN`].
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub down: bool,
    /// Auto-repeat while held.
    pub repeat: bool,
    pub native: NativeCode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestState {
    /// Keys held right now.
    pub held: BTreeSet<KeyCode>,
    /// Keys pressed at least once since the last reset, with their press count (repeats excluded).
    pub presses: BTreeMap<KeyCode, u32>,
    pub last: Option<KeyEvent>,
}

impl TestState {
    /// Applies one key event. Fails (and changes nothing) only when a new key would exceed
    /// [`MAX_KEYS`].
    pub fn apply(&mut self, mut event: KeyEvent) -> Result<(), DocError> {
        let known = self.presses.contains_key(&event.code) || self.held.contains(&event.code);
        if !known && self.presses.len().max(self.held.len()) >= MAX_KEYS {
            return Err(DocError::TooManyKeys);
        }
        if let Some(key) = &mut event.native.key {
            truncate_chars(key, MAX_KEY_TEXT_LEN);
        }
        if event.down {
            // A down for a key we already hold is a repeat, whatever the platform said.
            let fresh = self.held.insert(event.code.clone());
            event.repeat |= !fresh;
            if !event.repeat {
                let n = self.presses.entry(event.code.clone()).or_insert(0);
                *n = n.saturating_add(1);
            }
        } else {
            self.held.remove(&event.code);
        }
        self.last = Some(event);
        Ok(())
    }

    pub fn tested(&self, code: &KeyCode) -> bool {
        self.presses.contains_key(code)
    }
}

fn truncate_chars(s: &mut String, max: usize) {
    if let Some((i, _)) = s.char_indices().nth(max) {
        s.truncate(i);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct View {
    pub layout: String,
    pub theme: String,
}

impl Default for View {
    fn default() -> Self {
        Self {
            layout: "ansi-104".to_owned(),
            theme: "light".to_owned(),
        }
    }
}

/// Everything a session edits.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub test: TestState,
    pub view: View,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn ev(code: &str, down: bool) -> KeyEvent {
        KeyEvent {
            code: KeyCode::new(code).unwrap(),
            down,
            repeat: false,
            native: NativeCode::default(),
        }
    }

    #[test]
    fn code_validation() {
        assert!(KeyCode::new("NumpadEnter").is_ok());
        assert!(KeyCode::new("").is_err());
        assert!(KeyCode::new("Key A").is_err());
        assert!(KeyCode::new(&"A".repeat(MAX_CODE_LEN + 1)).is_err());
        assert!(serde_json::from_str::<KeyCode>("\"bad code\"").is_err());
    }

    #[test]
    fn press_counts_ignore_repeats() {
        let mut t = TestState::default();
        t.apply(ev("KeyA", true)).unwrap();
        t.apply(ev("KeyA", true)).unwrap(); // held → repeat
        t.apply(ev("KeyA", false)).unwrap();
        t.apply(ev("KeyA", true)).unwrap();
        let a = KeyCode::new("KeyA").unwrap();
        assert_eq!(t.presses.get(&a), Some(&2));
        assert!(t.held.contains(&a));
        assert!(t.last.as_ref().is_some_and(|e| !e.repeat));
    }

    #[test]
    fn key_cap() {
        let mut t = TestState::default();
        for i in 0..MAX_KEYS {
            t.apply(ev(&format!("K{i}"), true)).unwrap();
        }
        assert_eq!(t.apply(ev("Extra", true)), Err(DocError::TooManyKeys));
        assert!(t.apply(ev("K0", false)).is_ok());
    }

    proptest! {
        #[test]
        fn any_event_sequence_keeps_invariants(
            events in proptest::collection::vec(("[A-Za-z0-9]{1,4}", any::<bool>(), any::<bool>(), ".{0,80}"), 0..300)
        ) {
            let mut t = TestState::default();
            for (code, down, repeat, key) in events {
                let mut e = ev(&code, down);
                e.repeat = repeat;
                e.native.key = Some(key);
                let _ = t.apply(e);
                prop_assert!(t.held.len() <= MAX_KEYS && t.presses.len() <= MAX_KEYS);
                prop_assert!(t.presses.values().all(|n| *n >= 1));
                if let Some(k) = t.last.as_ref().and_then(|e| e.native.key.as_ref()) {
                    prop_assert!(k.chars().count() <= MAX_KEY_TEXT_LEN);
                }
            }
        }

        #[test]
        fn key_code_never_panics(s in ".{0,64}") {
            let _ = KeyCode::new(&s);
        }
    }
}
