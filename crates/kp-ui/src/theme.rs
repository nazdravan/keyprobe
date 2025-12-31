//! Themes: egui visuals plus the key palette, chosen by `View::theme` (`view.theme` command).

use egui::{Color32, Stroke, Theme, Visuals};

use crate::Palette;

/// `(id, menu title)`, in menu order; ids match `kp_engine::THEMES`.
pub const THEMES: &[(&str, &str)] = &[
    ("dark", "Dark"),
    ("light", "Light"),
    ("high-contrast", "High contrast"),
];

pub fn title(id: &str) -> &str {
    THEMES
        .iter()
        .find(|(t, _)| *t == id)
        .map_or(id, |(_, title)| title)
}

/// The egui base theme and visuals for a theme id; unknown ids fall back to dark.
pub fn visuals(id: &str) -> (Theme, Visuals) {
    match id {
        "light" => (Theme::Light, Visuals::light()),
        "high-contrast" => (Theme::Dark, high_contrast()),
        _ => (Theme::Dark, Visuals::dark()),
    }
}

/// Makes `id` the active theme. Each call sets the base theme's visuals explicitly, so leaving
/// high contrast restores the plain dark visuals.
pub fn apply(ctx: &egui::Context, id: &str) {
    let (base, v) = visuals(id);
    ctx.set_visuals_of(base, v);
    ctx.set_theme(base);
}

pub fn palette(id: &str, v: &Visuals) -> Palette {
    match id {
        "high-contrast" => Palette {
            untested_fill: Color32::BLACK,
            untested_text: Color32::WHITE,
            tested_fill: Color32::from_rgb(0, 230, 90),
            held_fill: Color32::from_rgb(255, 220, 0),
            marked_text: Color32::BLACK,
            border: Color32::WHITE,
            last: Color32::from_rgb(0, 200, 255),
        },
        _ => Palette::for_visuals(v),
    }
}

fn high_contrast() -> Visuals {
    let mut v = Visuals::dark();
    v.override_text_color = Some(Color32::WHITE);
    v.panel_fill = Color32::BLACK;
    v.window_fill = Color32::BLACK;
    v.extreme_bg_color = Color32::BLACK;
    v.faint_bg_color = Color32::from_gray(20);
    v.window_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.inactive.weak_bg_fill = Color32::BLACK;
    v.widgets.inactive.bg_fill = Color32::BLACK;
    v.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::from_rgb(0, 200, 255));
    v.selection.stroke = Stroke::new(2.0, Color32::from_rgb(0, 200, 255));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyState;

    #[test]
    fn ids_match_the_engine() {
        let ours: Vec<&str> = THEMES.iter().map(|(id, _)| *id).collect();
        let mut engine = kp_engine::THEMES.to_vec();
        let mut sorted = ours.clone();
        engine.sort_unstable();
        sorted.sort_unstable();
        assert_eq!(sorted, engine);
    }

    #[test]
    fn every_theme_keeps_states_distinct() {
        for (id, _) in THEMES {
            let (_, v) = visuals(id);
            let p = palette(id, &v);
            let fills = [
                p.fill(KeyState::Untested),
                p.fill(KeyState::Tested(1)),
                p.fill(KeyState::Held),
            ];
            assert_ne!(fills[0], fills[1], "{id}");
            assert_ne!(fills[1], fills[2], "{id}");
            assert_ne!(fills[0], fills[2], "{id}");
        }
    }

    #[test]
    fn leaving_high_contrast_restores_dark() {
        let ctx = egui::Context::default();
        apply(&ctx, "high-contrast");
        assert_eq!(ctx.global_style().visuals.panel_fill, Color32::BLACK);
        apply(&ctx, "dark");
        assert_eq!(ctx.global_style().visuals, Visuals::dark());
        apply(&ctx, "light");
        assert_eq!(ctx.global_style().visuals, Visuals::light());
    }
}
