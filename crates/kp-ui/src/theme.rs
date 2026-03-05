//! Themes: egui visuals plus the key palette, chosen by `View::theme` (`view.theme` command).

use egui::{Color32, Stroke, Theme, Visuals};

use crate::Palette;

/// Colours from `assets/theme.toml`; `brand_matches_theme_toml` keeps the two in sync.
pub mod brand {
    use egui::Color32;

    pub const ACCENT: Color32 = Color32::from_rgb(0x58, 0xbe, 0x6e);
    pub const ACCENT_PRESS: Color32 = Color32::from_rgb(0x46, 0xa3, 0x5a);
    pub const HELD: Color32 = Color32::from_rgb(0xf5, 0xb2, 0x32);
    pub const ALERT: Color32 = Color32::from_rgb(0xe6, 0x46, 0x3c);
    pub const INK: Color32 = Color32::from_rgb(0x14, 0x18, 0x16);
    pub const BG_DARK: Color32 = Color32::from_rgb(0x1b, 0x1b, 0x1b);
    pub const FG_DARK: Color32 = Color32::from_rgb(0xd4, 0xd8, 0xd5);
    pub const KEY_DARK: Color32 = Color32::from_rgb(0x3c, 0x3c, 0x3c);
    pub const SELECT_DARK: Color32 = Color32::from_rgb(0x2f, 0x6e, 0x3c);
    pub const BG_LIGHT: Color32 = Color32::from_rgb(0xf8, 0xf8, 0xf8);
    pub const FG_LIGHT: Color32 = Color32::from_rgb(0x26, 0x2a, 0x27);
    pub const KEY_LIGHT: Color32 = Color32::from_rgb(0xe4, 0xe4, 0xe4);
    pub const SELECT_LIGHT: Color32 = Color32::from_rgb(0xbf, 0xe6, 0xc8);
}

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
        "light" => (Theme::Light, light()),
        "high-contrast" => (Theme::Dark, high_contrast()),
        _ => (Theme::Dark, dark()),
    }
}

fn dark() -> Visuals {
    branded(
        Visuals::dark(),
        brand::BG_DARK,
        brand::FG_DARK,
        brand::KEY_DARK,
        brand::SELECT_DARK,
    )
}

fn light() -> Visuals {
    branded(
        Visuals::light(),
        brand::BG_LIGHT,
        brand::FG_LIGHT,
        brand::KEY_LIGHT,
        brand::SELECT_LIGHT,
    )
}

/// egui's base visuals with the theme's background, text, key and accent colours.
fn branded(mut v: Visuals, bg: Color32, fg: Color32, key: Color32, select: Color32) -> Visuals {
    v.panel_fill = bg;
    v.window_fill = bg;
    v.widgets.noninteractive.fg_stroke.color = fg;
    v.widgets.inactive.weak_bg_fill = key;
    v.hyperlink_color = brand::ACCENT;
    v.selection.bg_fill = select;
    v.selection.stroke = Stroke::new(2.0, brand::ACCENT);
    v.widgets.active.bg_stroke = Stroke::new(1.0, brand::ACCENT_PRESS);
    v
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
            alert: Color32::from_rgb(255, 60, 60),
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
                p.fill(KeyState::Stuck),
            ];
            for (i, a) in fills.iter().enumerate() {
                for b in fills.iter().skip(i + 1) {
                    assert_ne!(a, b, "{id}");
                }
            }
        }
    }

    #[test]
    fn leaving_high_contrast_restores_dark() {
        let ctx = egui::Context::default();
        apply(&ctx, "high-contrast");
        assert_eq!(ctx.global_style().visuals.panel_fill, Color32::BLACK);
        apply(&ctx, "dark");
        assert_eq!(ctx.global_style().visuals, dark());
        apply(&ctx, "light");
        assert_eq!(ctx.global_style().visuals, light());
    }

    /// Every `key = "#rrggbb"` in `assets/theme.toml` has the same value in `brand`.
    #[test]
    fn brand_matches_theme_toml() {
        let toml = include_str!("../../../assets/theme.toml");
        let ours = [
            ("accent", brand::ACCENT),
            ("accent_press", brand::ACCENT_PRESS),
            ("held", brand::HELD),
            ("alert", brand::ALERT),
            ("ink", brand::INK),
            ("bg_dark", brand::BG_DARK),
            ("fg_dark", brand::FG_DARK),
            ("key_dark", brand::KEY_DARK),
            ("select_dark", brand::SELECT_DARK),
            ("bg_light", brand::BG_LIGHT),
            ("fg_light", brand::FG_LIGHT),
            ("key_light", brand::KEY_LIGHT),
            ("select_light", brand::SELECT_LIGHT),
        ];
        let mut seen = 0;
        for line in toml.lines() {
            let Some((key, rest)) = line.split_once('=') else {
                continue;
            };
            let value = rest.split('#').nth(1).map(|h| &h[..6]);
            let Some(hex) = value.filter(|_| rest.trim_start().starts_with("\"#")) else {
                continue;
            };
            let key = key.trim();
            let (_, colour) = ours
                .iter()
                .find(|(k, _)| *k == key)
                .unwrap_or_else(|| panic!("theme.toml colour {key} missing from brand"));
            assert_eq!(colour.to_hex()[1..7], *hex, "{key}");
            seen += 1;
        }
        assert_eq!(seen, ours.len());
    }
}
