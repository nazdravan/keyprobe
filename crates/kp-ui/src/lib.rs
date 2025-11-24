//! L6: egui panels that draw the session and turn input into commands.
//!
//! Panels never touch the document: every action is `app.run(id, params)` (Rule 1).

use egui::{Button, RichText};
use kp_doc::KeyEvent;
use kp_engine::App;
use serde_json::Value;

/// Per-window UI state that is not part of the document.
#[derive(Debug, Default)]
pub struct KeyProbeUi {
    /// Last command error, shown until the next successful command.
    last_error: Option<String>,
}

impl KeyProbeUi {
    /// Draws the whole window.
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut App) {
        ui.heading("KeyProbe");
        self.toolbar(ui, app);
        ui.separator();
        readout(ui, app);
        if let Some(err) = &self.last_error {
            ui.colored_label(ui.visuals().error_fg_color, err);
        }
    }

    /// Runs a command and remembers its error for display.
    pub fn run(&mut self, app: &mut App, id: &str, params: &Value) {
        match app.run(id, params) {
            Ok(_) => self.last_error = None,
            Err(e) => self.last_error = Some(e.to_string()),
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, app: &mut App) {
        ui.horizontal(|ui| {
            let doc = app.session().doc();
            let (tested, total) = kp_keys::layout(&doc.view.layout).map_or((0, 0), |l| {
                let tested = l.keys.iter().filter(|k| doc.test.tested(&k.code)).count();
                (tested, l.keys.len())
            });
            ui.label(format!("Tested: {tested} / {total}"));
            for (id, label) in [
                ("test.reset", "Reset"),
                ("edit.undo", "Undo"),
                ("edit.redo", "Redo"),
            ] {
                let enabled = app.enabled(id);
                let button = ui.add_enabled(enabled.is_ok(), Button::new(label));
                let button = match &enabled {
                    Err(reason) => button.on_disabled_hover_text(reason.to_string()),
                    Ok(()) => button,
                };
                if button.clicked() {
                    self.run(app, id, &Value::Null);
                }
            }
        });
    }
}

/// The status readout: last key with derived and native codes.
fn readout(ui: &mut egui::Ui, app: &App) {
    let Some(event) = &app.session().doc().test.last else {
        ui.label("Press any key.");
        return;
    };
    egui::Grid::new("readout").num_columns(2).show(ui, |ui| {
        for (name, value) in readout_rows(event) {
            ui.label(name);
            ui.label(RichText::new(value).monospace());
            ui.end_row();
        }
    });
}

/// `(label, value)` rows for a key event; pure, so it is unit-tested without a UI.
pub fn readout_rows(event: &KeyEvent) -> Vec<(&'static str, String)> {
    let state = match (event.down, event.repeat) {
        (true, true) => "down (repeat)",
        (true, false) => "down",
        (false, _) => "up",
    };
    let mut rows = vec![("Key", format!("{} — {state}", event.code))];
    match kp_keys::info(&event.code) {
        Some(k) => {
            rows.push((
                "VK Code",
                format!("{}  {}", kp_keys::hex_dec(k.vk), k.vk_name),
            ));
            let ext = if k.extended { "  (Extended)" } else { "" };
            rows.push(("Scan Code", format!("{}{ext}", kp_keys::hex_dec(k.scan))));
        }
        None => rows.push(("VK / Scan", "not in key table".to_owned())),
    }
    if let Some(sc) = event.native.scancode {
        rows.push(("Native scancode", format!("0x{sc:X} ({sc})")));
    }
    if let Some(kc) = event.native.key_code {
        rows.push(("keyCode", kc.to_string()));
    }
    if let Some(key) = &event.native.key {
        rows.push(("Key value", format!("{key:?}")));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use kp_doc::{KeyCode, NativeCode};

    #[test]
    fn readout_for_print_screen_matches_reference_format() {
        let e = KeyEvent {
            code: KeyCode::new("PrintScreen").unwrap(),
            down: true,
            repeat: false,
            native: NativeCode {
                scancode: Some(0xE037),
                key_code: None,
                key: Some("PrintScreen".into()),
            },
        };
        let rows = readout_rows(&e);
        assert_eq!(rows[0], ("Key", "PrintScreen — down".to_owned()));
        assert_eq!(rows[1], ("VK Code", "0x2C (44)  VK_SNAPSHOT".to_owned()));
        assert_eq!(rows[2], ("Scan Code", "0x37 (55)  (Extended)".to_owned()));
        assert_eq!(rows[3], ("Native scancode", "0xE037 (57399)".to_owned()));
    }

    #[test]
    fn unknown_key_is_reported_not_hidden() {
        let e = KeyEvent {
            code: KeyCode::new("MediaPlayPause").unwrap(),
            down: false,
            repeat: false,
            native: NativeCode::default(),
        };
        let rows = readout_rows(&e);
        assert_eq!(rows[0].1, "MediaPlayPause — up");
        assert_eq!(rows[1], ("VK / Scan", "not in key table".to_owned()));
    }

    #[test]
    fn panel_renders_headless() {
        let mut app = App::default();
        let mut ui_state = KeyProbeUi::default();
        app.run("key.down", &serde_json::json!({"code": "KeyA"}))
            .unwrap();
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui_state.show(ui, &mut app);
        });
        // A headless run has no renderer to apply texture updates; drop them explicitly.
        out.textures_delta.clear();
        assert!(!out.shapes.is_empty());
        assert!(ui_state.last_error.is_none());
    }
}
