//! KeyProbe in the browser. eframe draws into `<canvas id="keyprobe">`; our own `keydown`/`keyup`
//! listeners on `window` (capture phase, so they run before eframe's) read the physical `code`,
//! `key` and legacy `keyCode`, which egui's `Key` would lose.
//!
//! Build: `cargo xtask web` → `dist/`.

pub mod keys;

#[cfg(target_arch = "wasm32")]
mod web;
