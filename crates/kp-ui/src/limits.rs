//! Keys the platform keeps for itself. A tester must say so, or users blame their keyboard.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOs,
    Linux,
    Web,
    Other,
}

impl Platform {
    /// The platform this build runs on, decided at compile time.
    pub const fn current() -> Self {
        if cfg!(target_arch = "wasm32") {
            Platform::Web
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            Platform::Other
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOs => "macOS",
            Platform::Linux => "Linux",
            Platform::Web => "the browser",
            Platform::Other => "this system",
        }
    }

    /// What can't reach KeyProbe here, in plain words.
    pub fn notes(self) -> &'static [&'static str] {
        match self {
            Platform::Windows => &[
                "Print Screen usually arrives as key-up only: Windows takes the key-down.",
                "Ctrl+Alt+Del, Win+L and other secure shortcuts never reach applications.",
                "Fn is handled inside the keyboard and is never reported.",
            ],
            Platform::MacOs => &[
                "Fn (Globe) is not reported as a key.",
                "Caps Lock reports a toggle: key-down when it turns on, key-up when it turns off.",
                "System shortcuts (Cmd+Tab, Cmd+Space, Cmd+Q…) are taken before KeyProbe sees them.",
                "Mac keyboards send F13–F15 where PC boards have Print Screen, Scroll Lock and Pause.",
            ],
            Platform::Linux => &[
                "Shortcuts bound by the desktop environment (Super, Alt+Tab, Print…) may not arrive.",
                "Fn is handled inside the keyboard and is never reported.",
            ],
            Platform::Web => &[
                "The browser keeps some shortcuts (Ctrl/Cmd+W, Ctrl/Cmd+T, Ctrl/Cmd+N, Cmd+Q).",
                "Print Screen and Fn are usually not delivered to web pages.",
                "Keys from virtual or IME keyboards without a physical code are ignored.",
                "Click the page first: keys go to the focused tab only.",
            ],
            Platform::Other => &["Some system shortcuts may not reach KeyProbe."],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_platform_has_notes() {
        for p in [
            Platform::Windows,
            Platform::MacOs,
            Platform::Linux,
            Platform::Web,
            Platform::Other,
        ] {
            assert!(!p.notes().is_empty(), "{p:?}");
            assert!(!p.name().is_empty());
        }
    }

    #[test]
    fn host_is_detected() {
        if cfg!(target_os = "macos") {
            assert_eq!(Platform::current(), Platform::MacOs);
        }
    }
}
