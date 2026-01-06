//! Browser key events → engine commands. Pure, so it is tested on the host.

use serde_json::{Map, Value, json};

/// Something the page saw that becomes a command on the next frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebInput {
    Key {
        code: String,
        down: bool,
        repeat: bool,
        key: Option<String>,
        key_code: Option<u32>,
        /// `event.timeStamp`, in whole ms.
        time_ms: Option<u64>,
    },
    /// The page lost focus: key-ups for held keys will go elsewhere.
    ReleaseAll,
}

/// Builds a key input from `KeyboardEvent` fields; `None` when the browser gives no physical
/// `code` (some virtual and IME keyboards), since there is no key to mark as tested.
pub fn key_input(
    code: &str,
    key: &str,
    key_code: u32,
    down: bool,
    repeat: bool,
    time_stamp: f64,
) -> Option<WebInput> {
    if code.is_empty() || code == "Unidentified" {
        return None;
    }
    Some(WebInput::Key {
        code: kp_keys::canonical(code).to_owned(),
        down,
        repeat,
        key: (!key.is_empty()).then(|| key.to_owned()),
        key_code: (key_code != 0).then_some(key_code),
        // f64 → u64 saturates; NaN and negatives (not produced by browsers) become None.
        time_ms: (time_stamp.is_finite() && time_stamp >= 0.0).then_some(time_stamp as u64),
    })
}

impl WebInput {
    pub fn command(&self) -> (&'static str, Value) {
        match self {
            WebInput::ReleaseAll => ("key.release_all", Value::Null),
            WebInput::Key {
                code,
                down,
                repeat,
                key,
                key_code,
                time_ms,
            } => {
                let mut native = Map::new();
                if let Some(k) = key {
                    native.insert("key".into(), json!(k));
                }
                if let Some(kc) = key_code {
                    native.insert("key_code".into(), json!(kc));
                }
                let mut params = Map::new();
                params.insert("code".into(), json!(code));
                params.insert("native".into(), Value::Object(native));
                if let Some(t) = time_ms {
                    params.insert("time_ms".into(), json!(t));
                }
                if *down {
                    params.insert("repeat".into(), json!(repeat));
                    ("key.down", Value::Object(params))
                } else {
                    ("key.up", Value::Object(params))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping() {
        assert!(key_input("", "a", 65, true, false, 0.0).is_none());
        assert!(key_input("Unidentified", "", 0, true, false, 0.0).is_none());
        let Some(WebInput::Key {
            code,
            key,
            key_code,
            ..
        }) = key_input("OSLeft", "Meta", 91, true, false, 0.0)
        else {
            panic!("expected a key");
        };
        assert_eq!(code, "MetaLeft");
        assert_eq!(key.as_deref(), Some("Meta"));
        assert_eq!(key_code, Some(91));
        let Some(WebInput::Key { time_ms, .. }) = key_input("KeyA", "a", 65, true, false, 1234.7)
        else {
            panic!("expected a key");
        };
        assert_eq!(time_ms, Some(1234));
        let Some(WebInput::Key { time_ms, .. }) = key_input("KeyA", "a", 65, true, false, f64::NAN)
        else {
            panic!("expected a key");
        };
        assert_eq!(time_ms, None);
    }

    #[test]
    fn commands_are_accepted_by_the_engine() {
        let mut app = kp_engine::App::default();
        let inputs = [
            key_input("KeyA", "a", 65, true, false, 0.0),
            key_input("KeyA", "a", 65, true, true, 0.0),
            key_input("NumpadEnter", "Enter", 13, true, false, 0.0),
            key_input("KeyA", "a", 65, false, false, 0.0),
            Some(WebInput::ReleaseAll),
        ];
        for input in inputs.into_iter().flatten() {
            let (id, params) = input.command();
            app.run(id, &params).unwrap();
        }
        let test = &app.session().doc().test;
        assert_eq!(test.presses.len(), 2);
        assert!(test.held.is_empty());
        let last = test.last.as_ref().unwrap();
        assert_eq!(last.native.key_code, Some(65));
    }

    #[test]
    fn every_ansi_code_round_trips() {
        let layout = kp_keys::layout("ansi-104").unwrap();
        for k in &layout.keys {
            let input = key_input(k.code.as_str(), "", 0, true, false, 0.0).unwrap();
            let (_, params) = input.command();
            assert_eq!(params["code"], k.code.as_str());
        }
    }
}
