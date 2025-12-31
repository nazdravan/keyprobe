//! Built-in commands. Every user-visible action is one of these (Rule 1).
//!
//! KeyProbe has no keyboard shortcuts by design: every key the user presses is input under test.
//! The `shortcut` field stays in the spec so a future non-keyboard trigger has one place to live.

use kp_doc::{KeyCode, KeyEvent, NativeCode};
use kp_keys::LAYOUT_NAMES;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{CommandSpec, EngineError, Registry, Session};

/// Themes the engine accepts for `view.theme` (grows with G9).
pub const THEMES: &[&str] = &["dark", "light", "high-contrast"];

pub fn builtin() -> Registry {
    let mut r = Registry::default();
    for spec in [
        CommandSpec {
            id: "key.down",
            label: "Key down",
            menu: None,
            shortcut: None,
            params: "{code: W3C code, repeat?: bool, native?: {scancode?, key_code?, key?}}",
            enabled: always,
            run: key_down,
        },
        CommandSpec {
            id: "key.up",
            label: "Key up",
            menu: None,
            shortcut: None,
            params: "{code: W3C code, native?: {scancode?, key_code?, key?}}",
            enabled: always,
            run: key_up,
        },
        CommandSpec {
            id: "key.release_all",
            label: "Release all keys",
            menu: None,
            shortcut: None,
            params: "{}",
            enabled: always,
            run: |s, p| {
                no_params("key.release_all", p)?;
                Ok(json!(s.release_all()))
            },
        },
        CommandSpec {
            id: "test.reset",
            label: "Reset",
            menu: Some("Test/Reset"),
            shortcut: None,
            params: "{}",
            enabled: |s| {
                if s.doc().test.presses.is_empty() {
                    Err("no keys tested yet".into())
                } else {
                    Ok(())
                }
            },
            run: |s, p| {
                no_params("test.reset", p)?;
                s.reset();
                Ok(Value::Null)
            },
        },
        CommandSpec {
            id: "view.layout",
            label: "Layout",
            menu: Some("View/Layout"),
            shortcut: None,
            params: "{name: one of kp_keys::LAYOUT_NAMES}",
            enabled: always,
            run: |s, p| {
                let name = choice("view.layout", p, LAYOUT_NAMES)?;
                s.set_view(|v| v.layout = name);
                Ok(Value::Null)
            },
        },
        CommandSpec {
            id: "view.theme",
            label: "Theme",
            menu: Some("View/Theme"),
            shortcut: None,
            params: "{name: one of THEMES}",
            enabled: always,
            run: |s, p| {
                let name = choice("view.theme", p, THEMES)?;
                s.set_view(|v| v.theme = name);
                Ok(Value::Null)
            },
        },
        CommandSpec {
            id: "edit.undo",
            label: "Undo",
            menu: Some("Edit/Undo"),
            shortcut: None,
            params: "{}",
            enabled: |s| {
                if s.history().can_undo() {
                    Ok(())
                } else {
                    Err("nothing to undo".into())
                }
            },
            run: |s, p| {
                no_params("edit.undo", p)?;
                Ok(json!(s.undo()))
            },
        },
        CommandSpec {
            id: "edit.redo",
            label: "Redo",
            menu: Some("Edit/Redo"),
            shortcut: None,
            params: "{}",
            enabled: |s| {
                if s.history().can_redo() {
                    Ok(())
                } else {
                    Err("nothing to redo".into())
                }
            },
            run: |s, p| {
                no_params("edit.redo", p)?;
                Ok(json!(s.redo()))
            },
        },
    ] {
        // Built-in ids are distinct; a duplicate here is caught by the `builtin_registry` test.
        let _ = r.register(spec);
    }
    r
}

fn always(_: &Session) -> Result<(), String> {
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyParams {
    code: KeyCode,
    #[serde(default)]
    repeat: bool,
    #[serde(default)]
    native: NativeCode,
}

fn key_down(s: &mut Session, p: &Value) -> Result<Value, EngineError> {
    key(s, "key.down", p, true)
}

fn key_up(s: &mut Session, p: &Value) -> Result<Value, EngineError> {
    key(s, "key.up", p, false)
}

fn key(s: &mut Session, id: &str, p: &Value, down: bool) -> Result<Value, EngineError> {
    let KeyParams {
        code,
        repeat,
        native,
    } = parse(id, p)?;
    s.key(KeyEvent {
        code,
        down,
        repeat: down && repeat,
        native,
    })?;
    Ok(Value::Null)
}

fn choice(id: &str, p: &Value, allowed: &[&str]) -> Result<String, EngineError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Name {
        name: String,
    }
    let Name { name } = parse(id, p)?;
    if allowed.contains(&name.as_str()) {
        Ok(name)
    } else {
        Err(EngineError::BadParams {
            id: id.to_owned(),
            message: format!(
                "unknown name {:?}; expected one of {allowed:?}",
                name.chars().take(64).collect::<String>()
            ),
        })
    }
}

fn no_params(id: &str, p: &Value) -> Result<(), EngineError> {
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Empty {}
    parse::<Empty>(id, p).map(|_| ())
}

/// Parses command params; `null` counts as `{}`.
fn parse<T: DeserializeOwned>(id: &str, p: &Value) -> Result<T, EngineError> {
    let p = if p.is_null() { json!({}) } else { p.clone() };
    serde_json::from_value(p).map_err(|e| EngineError::BadParams {
        id: id.to_owned(),
        message: e.to_string(),
    })
}
