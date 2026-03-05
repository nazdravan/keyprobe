//! The window icon: `assets/icon.png`, made by `assets/icon.py` from `assets/theme.toml`.

use std::io::Cursor;

const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon.png");

/// The decoded icon, or `None` (and the platform's default icon) if the PNG isn't 8-bit RGBA.
pub fn window_icon() -> Option<eframe::egui::IconData> {
    let mut decoder = png::Decoder::new(Cursor::new(ICON_PNG));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some(eframe::egui::IconData {
        rgba: buf,
        width: info.width,
        height: info.height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_decodes_to_rgba() {
        let icon = window_icon().unwrap();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }
}
