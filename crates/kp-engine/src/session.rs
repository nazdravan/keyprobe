//! The session: the document plus its undo history. Every edit goes through here.

use std::collections::{BTreeMap, VecDeque};

use kp_doc::{DocError, Document, KeyCode, KeyEvent, MAX_KEYS, View};

/// Undo steps kept; older ones are dropped.
pub const MAX_HISTORY: usize = 100;

/// One undoable edit, stored as what is needed to invert it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Edit {
    /// Press and chatter counts that the reset cleared. Undo adds them back on top of whatever
    /// was pressed since, so undoing a reset never loses newer test progress.
    Reset {
        presses: BTreeMap<KeyCode, u32>,
        chatter: BTreeMap<KeyCode, u32>,
    },
    View {
        before: View,
        after: View,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    undo: VecDeque<Edit>,
    redo: Vec<Edit>,
}

impl History {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn push(&mut self, edit: Edit) {
        if self.undo.len() >= MAX_HISTORY {
            self.undo.pop_front();
        }
        self.undo.push_back(edit);
        self.redo.clear();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    doc: Document,
    history: History,
}

impl Session {
    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    /// Key input. Not an edit: it is never undone (see docs/spec.md).
    pub fn key(&mut self, event: KeyEvent) -> Result<(), DocError> {
        self.doc.test.apply(event)
    }

    /// Input too: drops held keys whose key-up will never arrive. Not undoable.
    pub fn release_all(&mut self) -> usize {
        self.doc.test.release_all()
    }

    /// Clears tested keys and the last event; keys held right now stay held.
    /// Returns false (and records nothing) when there was nothing to clear.
    pub fn reset(&mut self) -> bool {
        let (presses, chatter) = self.clear_presses();
        if presses.is_empty() {
            return false;
        }
        self.history.push(Edit::Reset { presses, chatter });
        true
    }

    /// Changes the view; records history only if something actually changed.
    pub fn set_view(&mut self, change: impl FnOnce(&mut View)) -> bool {
        let before = self.doc.view.clone();
        change(&mut self.doc.view);
        if self.doc.view == before {
            return false;
        }
        let after = self.doc.view.clone();
        self.history.push(Edit::View { before, after });
        true
    }

    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.history.undo.pop_back() else {
            return false;
        };
        match &edit {
            Edit::Reset { presses, chatter } => {
                merge_counts(&mut self.doc.test.presses, presses);
                merge_counts(&mut self.doc.test.chatter, chatter);
            }
            Edit::View { before, .. } => self.doc.view = before.clone(),
        }
        self.history.redo.push(edit);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.history.redo.pop() else {
            return false;
        };
        let edit = match edit {
            // Redo is a fresh reset: it clears (and remembers) what is pressed now.
            Edit::Reset { .. } => {
                let (presses, chatter) = self.clear_presses();
                Edit::Reset { presses, chatter }
            }
            Edit::View { before, after } => {
                self.doc.view = after.clone();
                Edit::View { before, after }
            }
        };
        if self.history.undo.len() >= MAX_HISTORY {
            self.history.undo.pop_front();
        }
        self.history.undo.push_back(edit);
        true
    }

    /// Clears the test results (press and chatter counts, last event, release times) and returns
    /// the counts for undo. Held keys stay held.
    fn clear_presses(&mut self) -> (BTreeMap<KeyCode, u32>, BTreeMap<KeyCode, u32>) {
        let test = &mut self.doc.test;
        test.last = None;
        test.up_at.clear();
        (
            std::mem::take(&mut test.presses),
            std::mem::take(&mut test.chatter),
        )
    }
}

/// Adds `from` onto `into`, without letting `into` grow past [`MAX_KEYS`] keys.
fn merge_counts(into: &mut BTreeMap<KeyCode, u32>, from: &BTreeMap<KeyCode, u32>) {
    for (code, n) in from {
        if let Some(c) = into.get_mut(code) {
            *c = c.saturating_add(*n);
        } else if into.len() < MAX_KEYS {
            into.insert(code.clone(), *n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kp_doc::NativeCode;

    fn press(s: &mut Session, code: &str) {
        let code = KeyCode::new(code).unwrap();
        for down in [true, false] {
            s.key(KeyEvent {
                code: code.clone(),
                down,
                repeat: false,
                native: NativeCode::default(),
                time_ms: None,
            })
            .unwrap();
        }
    }

    fn count(s: &Session, code: &str) -> Option<u32> {
        s.doc()
            .test
            .presses
            .get(&KeyCode::new(code).unwrap())
            .copied()
    }

    #[test]
    fn undo_reset_keeps_newer_presses() {
        let mut s = Session::default();
        press(&mut s, "KeyA");
        press(&mut s, "KeyB");
        assert!(s.reset());
        press(&mut s, "KeyA");
        press(&mut s, "KeyC");
        assert!(s.undo());
        assert_eq!(count(&s, "KeyA"), Some(2));
        assert_eq!(count(&s, "KeyB"), Some(1));
        assert_eq!(count(&s, "KeyC"), Some(1));
        assert!(s.redo());
        assert!(s.doc().test.presses.is_empty());
        assert!(s.undo());
        assert_eq!(count(&s, "KeyA"), Some(2));
    }

    #[test]
    fn reset_clears_and_undo_restores_chatter() {
        let mut s = Session::default();
        let code = KeyCode::new("KeyA").unwrap();
        for (down, t) in [(true, 0), (false, 50), (true, 55), (false, 90)] {
            s.key(KeyEvent {
                code: code.clone(),
                down,
                repeat: false,
                native: NativeCode::default(),
                time_ms: Some(t),
            })
            .unwrap();
        }
        assert_eq!(s.doc().test.chatter.get(&code), Some(&1));
        assert!(s.reset());
        assert!(s.doc().test.chatter.is_empty());
        assert!(s.doc().test.up_at.is_empty());
        assert!(s.undo());
        assert_eq!(s.doc().test.chatter.get(&code), Some(&1));
    }

    #[test]
    fn empty_reset_is_not_recorded() {
        let mut s = Session::default();
        assert!(!s.reset());
        assert!(!s.history().can_undo());
    }

    #[test]
    fn view_undo_redo_and_noop() {
        let mut s = Session::default();
        assert!(!s.set_view(|v| v.theme = "dark".into()));
        assert!(s.set_view(|v| v.theme = "light".into()));
        assert!(s.undo());
        assert_eq!(s.doc().view.theme, "dark");
        assert!(s.redo());
        assert_eq!(s.doc().view.theme, "light");
        assert!(!s.redo());
    }

    #[test]
    fn history_is_capped() {
        let mut s = Session::default();
        for i in 0..(MAX_HISTORY + 20) {
            s.set_view(|v| v.theme = format!("t{i}"));
        }
        let mut undos = 0;
        while s.undo() {
            undos += 1;
        }
        assert_eq!(undos, MAX_HISTORY);
    }
}
