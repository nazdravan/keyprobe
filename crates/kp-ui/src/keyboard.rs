//! The keyboard panel: draws the active layout and colours every key by its test state.

use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};
use kp_doc::{Document, KeyCap, Layout, TestState};

/// Gap between neighbouring keys, in key units.
const GAP: f32 = 0.08;
/// Largest key size in points; beyond it the board stops growing.
const MAX_UNIT: f32 = 64.0;
/// Below this the legends are unreadable; the board scrolls instead of shrinking further.
const MIN_UNIT: f32 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Untested,
    /// Pressed at least once since the last reset, with the press count.
    Tested(u32),
    /// Held down right now.
    Held,
}

pub fn key_state(test: &TestState, key: &KeyCap) -> KeyState {
    if test.held.contains(&key.code) {
        KeyState::Held
    } else {
        test.presses
            .get(&key.code)
            .map_or(KeyState::Untested, |n| KeyState::Tested(*n))
    }
}

/// Points per key unit so that a board of `size` (in u) fits `avail` (in points).
pub fn unit_for(size: Vec2, avail: Vec2) -> f32 {
    if size.x <= 0.0 || size.y <= 0.0 {
        return MIN_UNIT;
    }
    let unit = (avail.x / size.x).min(avail.y / size.y);
    if unit.is_finite() {
        unit.clamp(MIN_UNIT, MAX_UNIT)
    } else {
        MIN_UNIT
    }
}

/// Colours for the three key states; tested/held fills are fixed so they read the same in light
/// and dark themes, and use dark text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub untested_fill: Color32,
    pub untested_text: Color32,
    pub tested_fill: Color32,
    pub held_fill: Color32,
    pub marked_text: Color32,
    pub border: Color32,
    pub last: Color32,
}

impl Palette {
    pub fn for_visuals(v: &egui::Visuals) -> Self {
        Self {
            untested_fill: v.widgets.inactive.weak_bg_fill,
            untested_text: v.text_color(),
            tested_fill: Color32::from_rgb(88, 190, 110),
            held_fill: Color32::from_rgb(245, 178, 50),
            marked_text: Color32::from_rgb(20, 24, 22),
            border: v.widgets.noninteractive.bg_stroke.color,
            last: v.selection.stroke.color,
        }
    }

    pub fn fill(&self, state: KeyState) -> Color32 {
        match state {
            KeyState::Untested => self.untested_fill,
            KeyState::Tested(_) => self.tested_fill,
            KeyState::Held => self.held_fill,
        }
    }

    fn text(&self, state: KeyState) -> Color32 {
        match state {
            KeyState::Untested => self.untested_text,
            _ => self.marked_text,
        }
    }
}

/// Draws `layout` with the state from `doc`, filling the available width.
pub fn show(ui: &mut egui::Ui, layout: &Layout, doc: &Document, palette: Palette, max_height: f32) {
    let Some(bounds) = layout.bounds() else {
        ui.label("Empty layout.");
        return;
    };
    let size = Vec2::new(bounds.right(), bounds.bottom());
    let unit = unit_for(size, Vec2::new(ui.available_width(), max_height));
    let (response, painter) = ui.allocate_painter(size * unit, Sense::hover());
    let origin = response.rect.min;
    let last = doc.test.last.as_ref().map(|e| &e.code);

    for key in &layout.keys {
        let state = key_state(&doc.test, key);
        let rect = key_rect(origin, unit, key);
        let radius = CornerRadius::same((unit * 0.12).clamp(2.0, 8.0) as u8);
        painter.rect_filled(rect, radius, palette.fill(state));
        let (stroke, kind) = if last == Some(&key.code) {
            (Stroke::new(2.0, palette.last), StrokeKind::Inside)
        } else {
            (Stroke::new(1.0, palette.border), StrokeKind::Inside)
        };
        painter.rect_stroke(rect, radius, stroke, kind);
        legends(&painter, rect, unit, key, palette.text(state));
        if let KeyState::Tested(n) = state
            && n > 1
        {
            painter.text(
                rect.right_bottom() - Vec2::splat(unit * 0.08),
                Align2::RIGHT_BOTTOM,
                n.to_string(),
                FontId::proportional(unit * 0.2),
                palette.marked_text,
            );
        }
    }

    if let Some(pos) = response.hover_pos() {
        let at = (pos - origin) / unit;
        if let Some(key) = layout.key_at(at.x, at.y) {
            response.on_hover_ui_at_pointer(|ui| tooltip(ui, key, &doc.test));
        }
    }
}

fn key_rect(origin: Pos2, unit: f32, key: &KeyCap) -> Rect {
    let r = key.rect;
    Rect::from_min_size(
        origin + Vec2::new(r.x + GAP / 2.0, r.y + GAP / 2.0) * unit,
        Vec2::new(r.w - GAP, r.h - GAP) * unit,
    )
}

fn legends(painter: &egui::Painter, rect: Rect, unit: f32, key: &KeyCap, color: Color32) {
    let pad = unit * 0.1;
    let room = rect.width() - 2.0 * pad;
    let main_size = legend_size(&key.label, unit);
    if key.shift_label.is_empty() {
        let font = fit(painter, &key.label, main_size, room);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            &key.label,
            font,
            color,
        );
    } else {
        let font = fit(
            painter,
            &key.shift_label,
            legend_size(&key.shift_label, unit) * 0.75,
            room,
        );
        painter.text(
            rect.left_top() + Vec2::splat(pad),
            Align2::LEFT_TOP,
            &key.shift_label,
            font,
            color,
        );
        let font = fit(painter, &key.label, main_size, room);
        painter.text(
            rect.left_bottom() + Vec2::new(pad, -pad),
            Align2::LEFT_BOTTOM,
            &key.label,
            font,
            color,
        );
    }
}

/// One-character legends (letters, digits, symbols, arrows) are big; words are smaller, all the
/// same size, so a row of modifiers reads evenly.
pub fn legend_size(text: &str, unit: f32) -> f32 {
    if text.chars().count() <= 1 {
        unit * 0.34
    } else {
        unit * 0.22
    }
}

/// Shrinks `size` until `text` fits in `max_width` points.
fn fit(painter: &egui::Painter, text: &str, size: f32, max_width: f32) -> FontId {
    let font = FontId::proportional(size);
    let width = painter
        .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
        .size()
        .x;
    if width > max_width && width > 0.0 && max_width > 0.0 {
        FontId::proportional(size * max_width / width)
    } else {
        font
    }
}

fn tooltip(ui: &mut egui::Ui, key: &KeyCap, test: &TestState) {
    ui.strong(key.code.as_str());
    if let Some(info) = kp_keys::info(&key.code) {
        ui.label(format!(
            "VK {}  {}",
            kp_keys::hex_dec(info.vk),
            info.vk_name
        ));
        let ext = if info.extended { "  (Extended)" } else { "" };
        ui.label(format!("Scan {}{ext}", kp_keys::hex_dec(info.scan)));
        ui.label(format!("USB HID 0x{:02X}", info.hid));
    }
    let presses = test.presses.get(&key.code).copied().unwrap_or(0);
    ui.label(format!("Pressed {presses}×"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use kp_doc::{KeyCode, KeyEvent, NativeCode};

    fn press(test: &mut TestState, code: &str, down: bool) {
        test.apply(KeyEvent {
            code: KeyCode::new(code).unwrap(),
            down,
            repeat: false,
            native: NativeCode::default(),
            time_ms: None,
        })
        .unwrap();
    }

    #[test]
    fn states() {
        let layout = kp_keys::layout("ansi-104").unwrap();
        let key = |c: &str| layout.key(&KeyCode::new(c).unwrap()).unwrap();
        let mut t = TestState::default();
        assert_eq!(key_state(&t, key("KeyA")), KeyState::Untested);
        press(&mut t, "KeyA", true);
        assert_eq!(key_state(&t, key("KeyA")), KeyState::Held);
        press(&mut t, "KeyA", false);
        press(&mut t, "KeyA", true);
        press(&mut t, "KeyA", false);
        assert_eq!(key_state(&t, key("KeyA")), KeyState::Tested(2));
    }

    #[test]
    fn legend_sizes_are_uniform_per_kind() {
        let u = 40.0;
        assert_eq!(legend_size("Ctrl", u), legend_size("Win", u));
        assert_eq!(legend_size("Esc", u), legend_size("Backspace", u));
        assert!(legend_size("A", u) > legend_size("Alt", u));
        assert_eq!(legend_size("←", u), legend_size("Q", u));
    }

    #[test]
    fn unit_scaling() {
        let board = Vec2::new(22.5, 6.5);
        assert_eq!(unit_for(board, Vec2::new(900.0, 1000.0)), 40.0);
        assert_eq!(unit_for(board, Vec2::new(9000.0, 9000.0)), MAX_UNIT);
        assert_eq!(unit_for(board, Vec2::new(10.0, 10.0)), MIN_UNIT);
        assert_eq!(unit_for(board, Vec2::new(f32::NAN, 100.0)), MIN_UNIT);
        assert_eq!(unit_for(Vec2::ZERO, Vec2::new(100.0, 100.0)), MIN_UNIT);
    }

    #[test]
    fn palette_states_are_distinct_in_both_themes() {
        for v in [egui::Visuals::light(), egui::Visuals::dark()] {
            let p = Palette::for_visuals(&v);
            let fills = [
                p.fill(KeyState::Untested),
                p.fill(KeyState::Tested(1)),
                p.fill(KeyState::Held),
            ];
            assert_ne!(fills[0], fills[1]);
            assert_ne!(fills[1], fills[2]);
            assert_ne!(fills[0], fills[2]);
        }
    }
}
