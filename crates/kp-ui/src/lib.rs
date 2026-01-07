//! L6: egui panels that draw the session and turn input into commands.
//!
//! Panels never touch the document: every action is `app.run(id, params)` (Rule 1).

mod keyboard;
mod limits;
mod theme;

use egui::{Button, RichText};
use kp_doc::KeyEvent;
use kp_engine::App;
use serde_json::{Value, json};

pub use keyboard::{KeyState, Palette, key_state, unit_for};
pub use limits::Platform;
pub use theme::THEMES;

/// Height kept free under the keyboard for the readout.
const READOUT_HEIGHT: f32 = 110.0;

/// Per-window UI state that is not part of the document.
#[derive(Debug, Default)]
pub struct KeyProbeUi {
    /// Last command error, shown until the next successful command.
    last_error: Option<String>,
    /// Theme id last applied to the egui context.
    applied_theme: Option<String>,
}

impl KeyProbeUi {
    /// Draws the whole window.
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut App) {
        self.apply_theme(ui.ctx(), &app.session().doc().view.theme);
        self.toolbar(ui, app);
        ui.separator();
        let doc = app.session().doc();
        match kp_keys::layout(&doc.view.layout) {
            Some(layout) => {
                let max_height = (ui.available_height() - READOUT_HEIGHT).max(0.0);
                let palette = theme::palette(&doc.view.theme, ui.visuals());
                keyboard::show(ui, &layout, doc, palette, max_height);
            }
            None => {
                ui.label(format!("Unknown layout {:?}", doc.view.layout));
            }
        }
        ui.separator();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width((ui.available_width() * 0.62).max(200.0));
                readout(ui, app);
            });
            ui.separator();
            ui.vertical(|ui| event_log(ui, app));
        });
        if let Some(err) = &self.last_error {
            ui.colored_label(ui.visuals().error_fg_color, err);
        }
    }

    /// Switches egui to the document's theme when it changed (undo can change it too).
    fn apply_theme(&mut self, ctx: &egui::Context, id: &str) {
        if self.applied_theme.as_deref() != Some(id) {
            theme::apply(ctx, id);
            self.applied_theme = Some(id.to_owned());
            // This frame's panel background is already drawn; paint again with the new one.
            ctx.request_repaint();
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
            ui.label(RichText::new(format!("Tested: {tested} / {total}")).strong());
            let current = doc.view.layout.clone();
            let title = |name: &str| kp_keys::layout(name).map_or(name.to_owned(), |l| l.title);
            let mut chosen = None;
            egui::ComboBox::from_id_salt("layout")
                .selected_text(title(&current))
                .show_ui(ui, |ui| {
                    for name in kp_keys::LAYOUT_NAMES {
                        if ui.selectable_label(current == *name, title(name)).clicked() {
                            chosen = Some(*name);
                        }
                    }
                });
            if let Some(name) = chosen {
                self.run(app, "view.layout", &json!({ "name": name }));
            }
            let current_theme = app.session().doc().view.theme.clone();
            let mut chosen_theme = None;
            egui::ComboBox::from_id_salt("theme")
                .selected_text(theme::title(&current_theme))
                .show_ui(ui, |ui| {
                    for (id, title) in THEMES {
                        if ui.selectable_label(current_theme == *id, *title).clicked() {
                            chosen_theme = Some(*id);
                        }
                    }
                });
            if let Some(name) = chosen_theme {
                self.run(app, "view.theme", &json!({ "name": name }));
            }
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
            ui.separator();
            let platform = Platform::current();
            ui.label(RichText::new("Platform limits").underline())
                .on_hover_ui(|ui| {
                    ui.strong(format!(
                        "Not every key reaches KeyProbe on {}:",
                        platform.name()
                    ));
                    for note in platform.notes() {
                        ui.label(format!("• {note}"));
                    }
                });
        });
    }
}

/// The status readout: last key with derived and native codes.
fn readout(ui: &mut egui::Ui, app: &App) {
    let Some(event) = &app.session().doc().test.last else {
        ui.label("Press any key.");
        return;
    };
    // Two label/value pairs per row keeps the readout short under the keyboard.
    let rows = readout_rows(event);
    egui::Grid::new("readout")
        .num_columns(4)
        .spacing([16.0, 4.0])
        .show(ui, |ui| {
            for pair in rows.chunks(2) {
                for (name, value) in pair {
                    ui.label(*name);
                    ui.label(RichText::new(value).monospace());
                }
                ui.end_row();
            }
        });
}

/// The event log, newest first, scrolling within the readout height.
fn event_log(ui: &mut egui::Ui, app: &App) {
    let lines = log_lines(&app.session().doc().test.log);
    if lines.is_empty() {
        ui.weak("Event log");
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(READOUT_HEIGHT - 10.0)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for line in lines {
                ui.label(RichText::new(line).monospace().small());
            }
        });
}

/// Log lines, newest first: `KeyA ↓ repeat  +12 ms` (time since the previous event, when both
/// events carry a timestamp).
pub fn log_lines(log: &std::collections::VecDeque<KeyEvent>) -> Vec<String> {
    let mut lines = Vec::with_capacity(log.len());
    let mut previous: Option<u64> = None;
    for e in log {
        let arrow = if e.down { "↓" } else { "↑" };
        let repeat = if e.repeat { " repeat" } else { "" };
        let delta = match (previous, e.time_ms) {
            (Some(p), Some(t)) => format!("  +{} ms", t.saturating_sub(p)),
            _ => String::new(),
        };
        lines.push(format!("{} {arrow}{repeat}{delta}", e.code));
        previous = e.time_ms;
    }
    lines.reverse();
    lines
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
            time_ms: None,
        };
        let rows = readout_rows(&e);
        assert_eq!(rows[0], ("Key", "PrintScreen — down".to_owned()));
        assert_eq!(rows[1], ("VK Code", "0x2C (44)  VK_SNAPSHOT".to_owned()));
        assert_eq!(rows[2], ("Scan Code", "0x37 (55)  (Extended)".to_owned()));
        assert_eq!(rows[3], ("Native scancode", "0xE037 (57399)".to_owned()));
    }

    #[test]
    fn log_lines_newest_first_with_deltas() {
        let mut app = App::default();
        for (id, code, t) in [
            ("key.down", "KeyA", 1000),
            ("key.down", "KeyA", 1500),
            ("key.up", "KeyA", 1520),
        ] {
            app.run(id, &serde_json::json!({"code": code, "time_ms": t}))
                .unwrap();
        }
        app.run("key.down", &serde_json::json!({"code": "KeyB"}))
            .unwrap();
        let lines = log_lines(&app.session().doc().test.log);
        assert_eq!(
            lines,
            [
                "KeyB ↓",
                "KeyA ↑  +20 ms",
                "KeyA ↓ repeat  +500 ms",
                "KeyA ↓"
            ]
        );
    }

    #[test]
    fn unknown_key_is_reported_not_hidden() {
        let e = KeyEvent {
            code: KeyCode::new("MediaPlayPause").unwrap(),
            down: false,
            repeat: false,
            native: NativeCode::default(),
            time_ms: None,
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
        app.run("key.down", &serde_json::json!({"code": "KeyB"}))
            .unwrap();
        app.run("key.up", &serde_json::json!({"code": "KeyB"}))
            .unwrap();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 500.0),
            )),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            ui_state.show(ui, &mut app);
        });
        // A headless run has no renderer to apply texture updates; drop them explicitly.
        out.textures_delta.clear();
        assert!(ui_state.last_error.is_none());

        fn fills(shape: &egui::Shape, out: &mut Vec<egui::Color32>) {
            match shape {
                egui::Shape::Rect(r) => out.push(r.fill),
                egui::Shape::Vec(v) => v.iter().for_each(|s| fills(s, out)),
                _ => {}
            }
        }
        let mut all = Vec::new();
        for clipped in &out.shapes {
            fills(&clipped.shape, &mut all);
        }
        let palette = theme::palette("dark", &ctx.global_style().visuals);
        let count = |c| all.iter().filter(|f| **f == c).count();
        assert_eq!(count(palette.fill(KeyState::Held)), 1, "KeyA held");
        assert_eq!(count(palette.fill(KeyState::Tested(1))), 1, "KeyB tested");
    }
}
