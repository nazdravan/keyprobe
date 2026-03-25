# Changelog

All notable changes to KeyProbe. Each version has a section `## [X.Y.Z] — YYYY-MM-DD` with
Added / Changed / Fixed; the section becomes the GitHub release notes (see `docs/releasing.md`).

## [0.1.0] — 2026-03-25

The first release.

### Added

- On-screen keyboard in five layouts: ANSI 104, ISO 105, TKL 87, 75 % and 60 %. Tested keys turn
  green, held keys amber, stuck keys red.
- Key readout: W3C `code`, Windows virtual-key code and name, PS/2 Set 1 scan code with the
  extended flag, the native scan code and key value; a note for layout-dependent keys; USB HID
  usage in the key tooltip. 138 keys in the key table.
- Event log of the last 50 events with the time between them.
- Chatter detection (a key pressed again less than 30 ms after its release) and stuck-key
  detection (a key held for more than 5 s).
- Held keys are released when the window or tab loses focus.
- Dark, light and high contrast themes; undo/redo for resets, layout and theme changes.
- Platform limits: the keys this OS or browser keeps for itself, in a toolbar tooltip.
- Web version (WebAssembly), built with `cargo xtask web`.
- Localhost control channel (`--control PORT`) to drive the desktop app and take screenshots.
- App icon, README banner and screenshots, all generated from `assets/theme.toml`.
