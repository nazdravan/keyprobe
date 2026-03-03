# KeyProbe

A keyboard tester for the desktop and the browser, written in Rust with [egui](https://github.com/emilk/egui).
Press every key: KeyProbe lights it up on an on-screen keyboard and shows its codes: W3C `code`,
Windows virtual-key code, PS/2 scan code, USB HID usage, and the raw values the system sent.

## Features

- **On-screen keyboard** in five layouts: ANSI 104, ISO 105, TKL 87, 75% and 60%.
  Untested keys are plain, tested keys are marked, held keys are highlighted.
- **Readout** for the last key: W3C `code`, Windows VK code and name, PS/2 Set 1 scan code
  (with the extended flag), the native scan code and key value the system sent, and a note for
  keys whose code depends on the active layout (non-US, Japanese, media keys). Hover a key on
  the keyboard to see its USB HID page and usage.
- **Event log** of the last 50 events: down/up, auto-repeat, time since the previous event.
- **Chatter detection**: a key pressed again less than 30 ms after its release gets a red badge
  and shows up in the toolbar summary.
- **Stuck-key detection**: a key held for more than 5 s turns red.
- **Focus-loss handling**: held keys are released when the window or tab loses focus, since their
  key-up events go elsewhere.
- **Themes**: dark, light and high contrast.
- **Undo/redo** for resets and view changes.
- **Platform limits**: a toolbar tooltip lists the keys this OS or browser keeps for itself
  (Print Screen on Windows, Fn everywhere, browser shortcuts…), so a missing key isn't blamed
  on the keyboard.

KeyProbe has no keyboard shortcuts: every key you press is input under test. The application uses
the mouse only.

## Running

Requires Rust 1.85 or newer (edition 2024). `rust-toolchain.toml` pins stable plus the
`wasm32-unknown-unknown` target, so rustup installs both.

### Desktop (macOS, Windows, Linux)

```sh
cargo run --release -p keyprobe
```

### Web

```sh
cargo install wasm-bindgen-cli --version 0.2.130   # once; must match Cargo.lock
cargo xtask web                                     # builds into dist/
python3 -m http.server 8765 -d dist
```

Open <http://localhost:8765/> and click the page so it has keyboard focus.

## What can't be tested

Some keys never reach an application, on any platform:

- **Fn** is handled inside the keyboard.
- **Secure shortcuts** (Ctrl+Alt+Del, Win+L) and system shortcuts (Cmd+Tab, Cmd+Space) are taken
  by the OS.
- **Print Screen on Windows** usually arrives as key-up only.
- **Caps Lock on macOS** reports a toggle, not separate down and up events.
- **In the browser**, the browser keeps its own shortcuts (Ctrl/Cmd+W, T, N), and keys from
  virtual or IME keyboards without a physical `code` are ignored.

The same list, for the platform you're on, is under **Platform limits** in the toolbar.

## Architecture

The workspace is split into layers. A crate depends only on lower layers, and nothing below the
UI layer touches egui, eframe or winit:

| Layer | Crate | Holds |
|---|---|---|
| L0 | `kp-geom` | rectangles and geometry checks |
| L1 | `kp-doc` | document model: key events, test state (presses, held, chatter), view, layout model |
| L2 | `kp-keys` | key table (138 keys: W3C ↔ HID ↔ VK ↔ Set 1) and layout data |
| L5 | `kp-engine` | session, command registry, undo/redo history, panic guard with rollback |
| L6 | `kp-ui` | egui panels: keyboard, readout, event log, toolbar, themes |
| L6 | `kp-control` | localhost control channel (desktop only) |
| L7 | `keyprobe-desktop` | eframe on its own winit event loop, raw key capture |
| L7 | `keyprobe-web` | eframe `WebRunner`, `keydown`/`keyup` listeners |

**Everything is a command.** The UI and the platform shells never change state directly; they call
`app.run(id, params)`:

| Command | Params |
|---|---|
| `key.down` | `{code, repeat?, native?: {scancode?, key_code?, key?}, time_ms?}` |
| `key.up` | `{code, native?, time_ms?}` |
| `key.release_all` | none |
| `test.reset` | none |
| `view.layout` | `{name: "ansi-104" \| "iso-105" \| "tkl-87" \| "75-percent" \| "60-percent"}` |
| `view.theme` | `{name: "dark" \| "light" \| "high-contrast"}` |
| `edit.undo`, `edit.redo` | none |
| `help.about` | none |

Commands never panic. Clippy denies `unwrap`, `expect`, `panic!` and slice indexing outside tests,
and a panic guard rolls the document back if a bug slips through.

### Key capture

- **Desktop:** eframe runs on KeyProbe's own winit event loop (`eframe::create_native`). A wrapper
  copies every `KeyboardInput` event (physical key and native scan code) before egui sees it,
  and egui gets no key events at all, so no key can trigger a UI action.
- **Web:** capture-phase `keydown`/`keyup` listeners on `window` call `preventDefault`, so Tab,
  Space, F5 and Alt reach the tester instead of the browser. `blur` releases held keys.

### Control channel

The desktop app can be driven over localhost for automated checks:

```sh
cargo run -p keyprobe -- --control 7878
```

Each request is one JSON line over TCP, and each reply is one line. Besides the commands above,
it accepts `ping`, `commands`, `state` and `ui.screenshot {path}`. The screenshot is a PNG of the
window, rendered by the app itself.

```sh
printf '%s\n' '{"id":1,"cmd":"key.down","params":{"code":"KeyA"}}' \
  '{"id":2,"cmd":"ui.screenshot","params":{"path":"/tmp/keyprobe.png"}}' | nc 127.0.0.1 7878
```

## Development

```sh
cargo xtask ci   # fmt, clippy -D warnings, tests, layer check, wasm clippy
```

Other tasks: `cargo xtask fmt | clippy | test | layers | wasm | web`.

### Sources

The key table comes only from public specifications:

- [W3C UI Events KeyboardEvent `code` values](https://www.w3.org/TR/uievents-code/)
- [USB HID Usage Tables](https://usb.org/document-library/hid-usage-tables-15)
- Microsoft's [Virtual-Key Codes](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes),
  [Keyboard Scan Code Specification](https://download.microsoft.com/download/1/6/1/161ba512-40e2-4cc9-843a-923143f3456c/scancode.doc)
  and [About Keyboard Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input)

KeyProbe is an original program. It contains no code from other keyboard testers.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
KeyProbe by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
