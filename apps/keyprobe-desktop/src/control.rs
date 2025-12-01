//! Desktop side of the control channel: CLI flag, server start, and the per-frame drain that
//! answers requests on the UI thread (screenshots come from eframe's own frame capture, so they
//! need no OS screen-recording permission).

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;

use eframe::egui;
use kp_control::{Action, Pending};
use serde_json::json;

/// Parses `--control PORT` from the command line. `Ok(None)` when absent.
pub fn port_from_args(args: impl IntoIterator<Item = String>) -> Result<Option<u16>, String> {
    let mut args = args.into_iter().skip(1);
    let mut port = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--control" => {
                let value = args.next().ok_or("--control needs a port")?;
                port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| format!("--control: bad port {value:?}"))?,
                );
            }
            other => {
                return Err(format!(
                    "unknown argument {other:?}; usage: keyprobe [--control PORT]"
                ));
            }
        }
    }
    Ok(port)
}

pub struct Control {
    rx: Receiver<Pending>,
    /// Screenshots requested but not yet read back.
    shots_in_flight: Arc<AtomicUsize>,
}

impl Control {
    /// Starts the server; the UI repaints whenever a request arrives.
    pub fn start(port: u16, ctx: &egui::Context) -> Option<Self> {
        let waker = ctx.clone();
        match kp_control::serve(port, Arc::new(move || waker.request_repaint())) {
            Ok((addr, rx)) => {
                eprintln!("keyprobe: control channel on {addr}");
                Some(Self {
                    rx,
                    shots_in_flight: Arc::default(),
                })
            }
            Err(e) => {
                eprintln!("keyprobe: control channel on port {port} failed: {e}");
                None
            }
        }
    }

    /// Answers every queued request. Call once per frame, before drawing.
    pub fn drain(&self, app: &mut kp_engine::App, ctx: &egui::Context) {
        while let Ok(pending) = self.rx.try_recv() {
            match kp_control::handle(app, &pending.request) {
                Ok(Action::Reply(value)) => pending.answer(Ok(value)),
                Err(e) => pending.answer(Err(e)),
                Ok(Action::Screenshot { path }) => {
                    // The callback runs once the next frame (which already shows this
                    // request's effects) has been read back.
                    let in_flight = Arc::clone(&self.shots_in_flight);
                    in_flight.fetch_add(1, Ordering::SeqCst);
                    ctx.request_screenshot(move |image| {
                        let [w, h] = image.size;
                        let result = save_png(&path, &image)
                            .map(|()| json!({"path": path, "width": w, "height": h}));
                        pending.answer(result);
                        in_flight.fetch_sub(1, Ordering::SeqCst);
                    });
                }
            }
        }
        // wgpu completes the read-back only when the device is polled, i.e. on a later frame;
        // an idle window paints no frames, so keep painting until every capture has arrived.
        if self.shots_in_flight.load(Ordering::SeqCst) > 0 {
            ctx.request_repaint();
        }
    }
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let [w, h] = image.size;
    let (Ok(w32), Ok(h32)) = (u32::try_from(w), u32::try_from(h)) else {
        return Err(format!("screenshot too large: {w}×{h}"));
    };
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), w32, h32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let data: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|c| c.to_srgba_unmultiplied())
        .collect();
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(&data))
        .map_err(|e| format!("png: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        std::iter::once("keyprobe")
            .chain(list.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn cli() {
        assert_eq!(port_from_args(args(&[])), Ok(None));
        assert_eq!(port_from_args(args(&["--control", "7878"])), Ok(Some(7878)));
        assert!(port_from_args(args(&["--control"])).is_err());
        assert!(port_from_args(args(&["--control", "99999"])).is_err());
        assert!(port_from_args(args(&["--bogus"])).is_err());
    }

    #[test]
    fn png_round_trip() {
        let dir = std::env::temp_dir().join(format!("kp-png-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.png");
        let image = egui::ColorImage::new([3, 2], vec![egui::Color32::RED; 6]);
        save_png(&path, &image).unwrap();
        let decoder = png::Decoder::new(BufReader::new(File::open(&path).unwrap()));
        let reader = decoder.read_info().unwrap();
        assert_eq!((reader.info().width, reader.info().height), (3, 2));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    use std::io::BufReader;
}
