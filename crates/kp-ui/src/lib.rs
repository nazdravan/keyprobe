//! L6: egui panels that draw the session and turn input into commands.
//!
//! Panels never touch the document: every action is `app.run(id, params)` (Rule 1).

mod keyboard;
mod limits;
mod theme;

use egui::{Button, Color32, RichText, TextStyle};
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
    /// Current time on the same clock as the key events' `time_ms` (set by the app each frame).
    now_ms: Option<u64>,
    /// The About dialog's content while it is open (the result of `help.about`).
    about: Option<Value>,
    /// `Some(enabled)`: the About dialog offers "Check for updates…" (desktop only; the web
    /// build has nothing to update).
    update_button: Option<bool>,
    /// "Check for updates…" was clicked and the app has not picked it up yet.
    update_requested: bool,
}

impl KeyProbeUi {
    /// Draws the whole window.
    pub fn show(&mut self, ui: &mut egui::Ui, app: &mut App) {
        self.apply_theme(ui.ctx(), &app.session().doc().view.theme);
        if !app.session().doc().test.held.is_empty() {
            // Held keys can turn "stuck" without any new event; keep the clock moving.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
        self.toolbar(ui, app);
        ui.separator();
        let doc = app.session().doc();
        let unit = match kp_keys::layout(&doc.view.layout) {
            Some(layout) => {
                let max_height = (ui.available_height() - READOUT_HEIGHT).max(0.0);
                let palette = theme::palette(&doc.view.theme, ui.visuals());
                keyboard::show(ui, &layout, doc, palette, self.now_ms, max_height)
            }
            None => {
                ui.label(format!("Unknown layout {:?}", doc.view.layout));
                0.0
            }
        };
        ui.separator();
        ui.horizontal_top(|ui| {
            let size = keyboard::panel_text_size(unit);
            for style in [TextStyle::Body, TextStyle::Monospace] {
                if let Some(font) = ui.style_mut().text_styles.get_mut(&style) {
                    font.size = size;
                }
            }
            ui.vertical(|ui| {
                ui.set_width((ui.available_width() * 0.5).max(200.0));
                readout(ui, app);
            });
            ui.separator();
            ui.vertical(|ui| event_log(ui, app));
        });
        self.about_window(ui.ctx());
        if let Some(err) = &self.last_error {
            ui.colored_label(ui.visuals().error_fg_color, err);
        }
    }

    /// Sets the current time, on the clock the app stamps key events with. Enables stuck-key
    /// detection; call before [`Self::show`] every frame.
    pub fn set_clock(&mut self, now_ms: u64) {
        self.now_ms = Some(now_ms);
    }

    /// Offers "Check for updates…" in the About dialog (`Some(enabled)`), or hides it (`None`).
    pub fn set_update_button(&mut self, button: Option<bool>) {
        self.update_button = button;
    }

    /// Whether "Check for updates…" was clicked since the last call.
    pub fn take_update_request(&mut self) -> bool {
        std::mem::take(&mut self.update_requested)
    }

    /// Shows `message` where command errors go, until the next successful command.
    pub fn set_error(&mut self, message: String) {
        self.last_error = Some(message);
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

    fn about_window(&mut self, ctx: &egui::Context) {
        let Some(about) = &self.about else {
            return;
        };
        let text = |key: &str| about[key].as_str().unwrap_or_default().to_owned();
        let mut open = true;
        egui::Window::new("About KeyProbe")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading(format!("{} {}", text("name"), text("version")));
                ui.label(text("description"));
                ui.label(format!("License: {}", text("license")));
                ui.hyperlink(text("repository"));
                ui.separator();
                ui.label("Key codes from public specifications:");
                for s in about["sources"].as_array().into_iter().flatten() {
                    ui.label(format!("• {}", s.as_str().unwrap_or_default()));
                }
                if let Some(enabled) = self.update_button {
                    ui.separator();
                    if ui
                        .add_enabled(enabled, Button::new("Check for updates…"))
                        .clicked()
                    {
                        self.update_requested = true;
                    }
                }
            });
        if !open || self.update_requested {
            self.about = None;
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
            if let Some(summary) = chatter_summary(&doc.test) {
                ui.colored_label(Color32::from_rgb(230, 70, 60), summary)
                    .on_hover_text(format!(
                        "Pressed again less than {} ms after release: the switch may bounce.",
                        kp_doc::CHATTER_MS
                    ));
            }
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
            if ui.button("About…").clicked() {
                match app.run("help.about", &Value::Null) {
                    Ok(about) => self.about = Some(about),
                    Err(e) => self.last_error = Some(e.to_string()),
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

/// `Chatter: KeyA ×2, KeyF ×1`, or `None` when no key chattered.
pub fn chatter_summary(test: &kp_doc::TestState) -> Option<String> {
    if test.chatter.is_empty() {
        return None;
    }
    let keys: Vec<String> = test
        .chatter
        .iter()
        .map(|(code, n)| format!("{code} ×{n}"))
        .collect();
    Some(format!("Chatter: {}", keys.join(", ")))
}

/// The event log, newest first, filling the height left under the keyboard.
fn event_log(ui: &mut egui::Ui, app: &App) {
    let lines = log_lines(&app.session().doc().test.log);
    if lines.is_empty() {
        ui.weak("Event log");
        return;
    }
    egui::ScrollArea::vertical()
        .max_height(ui.available_height())
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for line in lines {
                ui.label(RichText::new(line).monospace());
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
            if !k.note.is_empty() {
                rows.push(("Note", k.note.to_owned()));
            }
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
    fn readout_for_print_screen() {
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
    fn update_button_only_when_offered() {
        let mut app = App::default();
        let about = app.run("help.about", &Value::Null).unwrap();
        for offered in [false, true] {
            let mut ui_state = KeyProbeUi {
                about: Some(about.clone()),
                ..Default::default()
            };
            if offered {
                ui_state.set_update_button(Some(true));
            }
            let ctx = egui::Context::default();
            let mut found = false;
            // A new window is laid out invisibly on its first frame.
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui_state.show(ui, &mut app);
            });
            out.textures_delta.clear();
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui_state.show(ui, &mut app);
            });
            out.textures_delta.clear();
            for shape in &out.shapes {
                if let egui::epaint::Shape::Text(t) = &shape.shape {
                    found |= t.galley.text().contains("Check for updates");
                }
            }
            assert_eq!(found, offered);
        }
        let mut ui_state = KeyProbeUi {
            update_requested: true,
            ..Default::default()
        };
        assert!(ui_state.take_update_request());
        assert!(!ui_state.take_update_request());
    }

    #[test]
    fn about_window_renders_headless() {
        let mut app = App::default();
        let mut ui_state = KeyProbeUi {
            about: Some(app.run("help.about", &Value::Null).unwrap()),
            ..Default::default()
        };
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            ui_state.show(ui, &mut app);
        });
        out.textures_delta.clear();
        assert!(ui_state.about.is_some());
    }

    #[test]
    fn chatter_summary_lists_keys() {
        let mut app = App::default();
        assert!(chatter_summary(&app.session().doc().test).is_none());
        for (id, t) in [("key.down", 0), ("key.up", 40), ("key.down", 45)] {
            app.run(id, &serde_json::json!({"code": "KeyF", "time_ms": t}))
                .unwrap();
        }
        assert_eq!(
            chatter_summary(&app.session().doc().test).as_deref(),
            Some("Chatter: KeyF ×1")
        );
    }

    #[test]
    fn layout_dependent_keys_carry_their_note() {
        let e = KeyEvent {
            code: KeyCode::new("IntlYen").unwrap(),
            down: true,
            repeat: false,
            native: NativeCode::default(),
            time_ms: None,
        };
        let rows = readout_rows(&e);
        assert!(
            rows.iter()
                .any(|(name, v)| *name == "Note" && v.contains("Japanese"))
        );
    }

    #[test]
    fn unknown_key_is_reported_not_hidden() {
        let e = KeyEvent {
            code: KeyCode::new("LaunchApp1").unwrap(),
            down: false,
            repeat: false,
            native: NativeCode::default(),
            time_ms: None,
        };
        let rows = readout_rows(&e);
        assert_eq!(rows[0].1, "LaunchApp1 — up");
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
