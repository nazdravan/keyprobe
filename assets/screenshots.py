"""README screenshots, made by the app itself through its control channel (never by hand).

    cargo build --release -p keyprobe
    python3 assets/screenshots.py [path/to/keyprobe]

Starts KeyProbe with `--control`, plays scripted key events and saves, in docs/assets/:
  screenshot-main.png              dark, ANSI 104, mid-test: tested keys, a held key, chatter
  screenshot-light-iso.png         light theme, ISO 105, a non-US key in the readout
  screenshot-high-contrast-60.png  high contrast theme, 60 % layout
All are taken at the app's default window size, so they can be regenerated after UI changes.
"""
import json
import pathlib
import socket
import subprocess
import sys
import time

import theme

OUT = theme.ROOT / "docs" / "assets"


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


class App:
    def __init__(self, binary):
        self.port = free_port()
        self.proc = subprocess.Popen([str(binary), "--control", str(self.port)])
        deadline = time.monotonic() + 30
        while True:
            try:
                self.sock = socket.create_connection(("127.0.0.1", self.port), timeout=15)
                break
            except OSError:
                if time.monotonic() > deadline or self.proc.poll() is not None:
                    raise SystemExit("keyprobe did not open its control port")
                time.sleep(0.2)
        self.file = self.sock.makefile("rw")
        self.next_id = 0

    def run(self, cmd, **params):
        self.next_id += 1
        self.file.write(json.dumps({"id": self.next_id, "cmd": cmd, "params": params}) + "\n")
        self.file.flush()
        reply = json.loads(self.file.readline())
        if not reply.get("ok"):
            raise SystemExit(f"{cmd} {params}: {reply}")
        return reply.get("result")

    def tap(self, code, t, hold=60):
        self.run("key.down", code=code, time_ms=t)
        self.run("key.up", code=code, time_ms=t + hold)

    def shot(self, name):
        path = OUT / name
        self.run("ui.screenshot", path=str(path))
        print(path)

    def close(self):
        self.sock.close()
        self.proc.terminate()
        self.proc.wait(timeout=10)


def main():
    binary = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else theme.ROOT / "target" / "release" / "keyprobe"
    OUT.mkdir(parents=True, exist_ok=True)
    app = App(binary)
    try:
        # Event times are milliseconds on the app's clock, which starts with the app.
        t = 0
        for code in ["Escape", "Backquote", "Digit1", "Digit2", "Digit3", "Tab", "KeyQ", "KeyW", "KeyE",
                     "KeyR", "CapsLock", "KeyA", "KeyS", "KeyD", "KeyK", "KeyP", "Space", "ArrowLeft",
                     "ArrowUp", "ArrowDown", "ArrowRight", "Numpad5", "F1", "F2", "F3", "F4"]:
            t += 90
            app.tap(code, t)
        # A bouncing switch: F pressed again 18 ms after its release.
        app.run("key.down", code="KeyF", time_ms=t + 100)
        app.run("key.up", code="KeyF", time_ms=t + 160)
        app.run("key.down", code="KeyF", time_ms=t + 178)
        app.run("key.up", code="KeyF", time_ms=t + 240)
        # Held now (stamped with the current time), with the codes a real keyboard sends.
        app.run("key.down", code="ShiftLeft", native={"scancode": 42, "key_code": 16, "key": "Shift"})
        app.shot("screenshot-main.png")

        app.run("key.release_all")
        app.run("test.reset")
        app.run("view.layout", name="iso-105")
        app.run("view.theme", name="light")
        t = 0
        for code in ["KeyZ", "KeyX", "KeyC", "Enter", "Quote", "Backslash", "ShiftRight"]:
            t += 90
            app.tap(code, t)
        app.run("key.down", code="IntlBackslash", native={"scancode": 86, "key_code": 226, "key": "\\"})
        app.run("key.up", code="IntlBackslash")
        app.shot("screenshot-light-iso.png")

        app.run("test.reset")
        app.run("view.layout", name="60-percent")
        app.run("view.theme", name="high-contrast")
        t = 0
        for code in ["KeyH", "KeyI", "Space", "KeyC", "KeyO", "KeyN", "KeyT", "KeyR", "KeyA", "KeyS"]:
            t += 90
            app.tap(code, t)
        app.run("key.down", code="AltLeft")
        app.shot("screenshot-high-contrast-60.png")
    finally:
        app.close()


if __name__ == "__main__":
    main()
