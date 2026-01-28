//! L5: session, history, command registry and panic guard.
//!
//! UI, CLI and the control channel all do the same thing: `app.run("test.reset", params)`.

mod commands;
mod session;

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};

use kp_doc::DocError;
use serde_json::Value;

pub use commands::{THEMES, builtin};
pub use session::{History, MAX_HISTORY, Session};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("unknown command {0:?}")]
    UnknownCommand(String),
    #[error("command {id:?} already registered")]
    Duplicate { id: String },
    #[error("{id} is disabled: {reason}")]
    Disabled { id: String, reason: String },
    #[error("{id}: bad params: {message}")]
    BadParams { id: String, message: String },
    #[error(transparent)]
    Doc(#[from] DocError),
    #[error("{id} crashed and was rolled back: {message}")]
    Panicked { id: String, message: String },
}

pub type EnabledFn = fn(&Session) -> Result<(), String>;
pub type RunFn = fn(&mut Session, &Value) -> Result<Value, EngineError>;

#[derive(Clone, Copy)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    /// Menu path such as `"Test/Reset"`; `None` for input-only commands.
    pub menu: Option<&'static str>,
    pub shortcut: Option<&'static str>,
    /// Human-readable params doc.
    pub params: &'static str,
    pub enabled: EnabledFn,
    pub run: RunFn,
}

#[derive(Clone, Default)]
pub struct Registry {
    commands: BTreeMap<&'static str, CommandSpec>,
}

impl Registry {
    pub fn register(&mut self, spec: CommandSpec) -> Result<(), EngineError> {
        if self.commands.contains_key(spec.id) {
            return Err(EngineError::Duplicate {
                id: spec.id.to_owned(),
            });
        }
        self.commands.insert(spec.id, spec);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&CommandSpec> {
        self.commands.get(id)
    }

    pub fn specs(&self) -> impl Iterator<Item = &CommandSpec> {
        self.commands.values()
    }
}

/// The engine entry point: one session, one registry.
#[derive(Clone)]
pub struct App {
    session: Session,
    registry: Registry,
}

impl Default for App {
    fn default() -> Self {
        Self::new(builtin())
    }
}

impl App {
    pub fn new(registry: Registry) -> Self {
        Self {
            session: Session::default(),
            registry,
        }
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Why `id` can't run right now, or `Ok` if it can.
    pub fn enabled(&self, id: &str) -> Result<(), EngineError> {
        let spec = self.spec(id)?;
        (spec.enabled)(&self.session).map_err(|reason| EngineError::Disabled {
            id: id.to_owned(),
            reason,
        })
    }

    /// Runs a command atomically: on error or panic the session is rolled back to how it was.
    /// The panic guard is a safety net for bugs (wasm aborts on panic, so it is not a substitute
    /// for returning `Result`).
    pub fn run(&mut self, id: &str, params: &Value) -> Result<Value, EngineError> {
        self.enabled(id)?;
        let run = self.spec(id)?.run;
        let before = self.session.clone();
        let result = catch_unwind(AssertUnwindSafe(|| run(&mut self.session, params)))
            .unwrap_or_else(|payload| {
                Err(EngineError::Panicked {
                    id: id.to_owned(),
                    message: panic_message(payload.as_ref()),
                })
            });
        if result.is_err() {
            self.session = before;
        }
        result
    }

    fn spec(&self, id: &str) -> Result<&CommandSpec, EngineError> {
        self.registry
            .get(id)
            .ok_or_else(|| EngineError::UnknownCommand(id.chars().take(64).collect()))
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kp_doc::KeyCode;
    use proptest::prelude::*;
    use serde_json::json;

    fn tested(app: &App, code: &str) -> bool {
        app.session()
            .doc()
            .test
            .tested(&KeyCode::new(code).unwrap())
    }

    #[test]
    fn builtin_registry() {
        let r = builtin();
        let ids: Vec<_> = r.specs().map(|s| s.id).collect();
        for id in [
            "key.down",
            "key.up",
            "key.release_all",
            "test.reset",
            "view.layout",
            "view.theme",
            "edit.undo",
            "edit.redo",
            "help.about",
        ] {
            assert!(ids.contains(&id), "missing {id}");
        }
        let mut r = r;
        let spec = *r.get("key.down").unwrap();
        assert!(matches!(
            r.register(spec),
            Err(EngineError::Duplicate { .. })
        ));
    }

    #[test]
    fn default_and_selectable_layouts_exist() {
        let app = App::default();
        assert!(kp_keys::layout(&app.session().doc().view.layout).is_some());
        let mut app = app;
        for name in kp_keys::LAYOUT_NAMES {
            app.run("view.layout", &json!({ "name": name })).unwrap();
            assert_eq!(&app.session().doc().view.layout, name);
        }
    }

    #[test]
    fn about_is_a_command() {
        let mut app = App::default();
        let before = app.session().clone();
        let about = app.run("help.about", &Value::Null).unwrap();
        assert_eq!(about["name"], "KeyProbe");
        assert_eq!(about["version"], env!("CARGO_PKG_VERSION"));
        assert!(about["sources"].as_array().is_some_and(|s| !s.is_empty()));
        assert_eq!(app.session(), &before);
    }

    #[test]
    fn release_all_through_commands() {
        let mut app = App::default();
        app.run("key.down", &json!({"code": "KeyA"})).unwrap();
        assert_eq!(app.run("key.release_all", &Value::Null), Ok(json!(1)));
        assert!(app.session().doc().test.held.is_empty());
        assert!(tested(&app, "KeyA"));
        assert!(!app.session().history().can_undo());
    }

    #[test]
    fn key_reset_undo_through_commands() {
        let mut app = App::default();
        assert!(matches!(
            app.run("test.reset", &Value::Null),
            Err(EngineError::Disabled { .. })
        ));
        app.run(
            "key.down",
            &json!({"code": "KeyA", "native": {"scancode": 30}}),
        )
        .unwrap();
        app.run("key.up", &json!({"code": "KeyA"})).unwrap();
        assert!(tested(&app, "KeyA"));
        app.run("test.reset", &Value::Null).unwrap();
        assert!(!tested(&app, "KeyA"));
        app.run("edit.undo", &json!({})).unwrap();
        assert!(tested(&app, "KeyA"));
    }

    #[test]
    fn bad_params_are_errors_not_changes() {
        let mut app = App::default();
        for (id, p) in [
            ("key.down", json!({})),
            ("key.down", json!({"code": "no spaces"})),
            ("key.down", json!({"code": "KeyA", "extra": 1})),
            ("view.theme", json!({"name": "neon"})),
            ("view.layout", json!(42)),
            ("test.reset", json!({"x": 1})),
        ] {
            let before = app.session().clone();
            assert!(app.run(id, &p).is_err(), "{id} {p}");
            assert_eq!(app.session(), &before);
        }
        assert!(matches!(
            app.run("nope", &Value::Null),
            Err(EngineError::UnknownCommand(_))
        ));
    }

    #[test]
    fn panic_guard_rolls_back() {
        let mut r = builtin();
        r.register(CommandSpec {
            id: "test.crash",
            label: "Crash",
            menu: None,
            shortcut: None,
            params: "{}",
            enabled: |_| Ok(()),
            run: |s, _| {
                s.set_view(|v| v.theme = "dark".into());
                panic!("boom");
            },
        })
        .unwrap();
        let mut app = App::new(r);
        let before = app.session().clone();
        let err = app.run("test.crash", &Value::Null).unwrap_err();
        assert_eq!(
            err,
            EngineError::Panicked {
                id: "test.crash".into(),
                message: "boom".into()
            }
        );
        assert_eq!(app.session(), &before);
    }

    fn arb_json() -> impl Strategy<Value = Value> {
        let leaf = prop_oneof![
            Just(Value::Null),
            any::<bool>().prop_map(Value::from),
            any::<i64>().prop_map(Value::from),
            "[A-Za-z0-9 ]{0,20}".prop_map(Value::from),
            Just(json!("KeyA")),
            Just(json!("dark")),
        ];
        leaf.prop_recursive(3, 24, 4, |inner| {
            prop_oneof![
                proptest::collection::vec(inner.clone(), 0..4).prop_map(Value::from),
                proptest::collection::btree_map(
                    prop_oneof![
                        Just("code".to_owned()),
                        Just("name".to_owned()),
                        Just("repeat".to_owned()),
                        Just("native".to_owned()),
                        Just("scancode".to_owned()),
                        Just("key".to_owned()),
                        "[a-z]{1,6}",
                    ],
                    inner,
                    0..4
                )
                .prop_map(|m| Value::Object(m.into_iter().collect())),
            ]
        })
    }

    proptest! {
        /// Untrusted params to any command: never a panic; failures leave the session untouched.
        #[test]
        fn any_params_any_command(steps in proptest::collection::vec((0usize..64, arb_json()), 0..60)) {
            let mut app = App::default();
            let ids: Vec<&str> = app.registry().specs().map(|s| s.id).chain(["bogus"]).collect();
            for (i, params) in steps {
                let id = ids[i % ids.len()];
                let before = app.session().clone();
                match app.run(id, &params) {
                    Ok(_) => {}
                    Err(EngineError::Panicked { .. }) => prop_assert!(false, "{id} panicked on {params}"),
                    Err(_) => prop_assert_eq!(app.session(), &before),
                }
            }
        }
    }
}
