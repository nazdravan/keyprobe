//! KeyProbe desktop: eframe draws, but runs on our own winit event loop so we can see every raw
//! `KeyboardInput` (physical key + platform scancode) before egui turns it into a lossy `Key`.

mod capture;
mod control;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use eframe::{UserEvent, egui};
use kp_ui::KeyProbeUi;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use capture::RawKey;
use control::Control;

/// Captured keys not yet turned into commands; bounded so a stalled UI can't grow it forever.
const MAX_QUEUED_KEYS: usize = 1024;

type KeyQueue = Arc<Mutex<VecDeque<RawKey>>>;

fn main() -> eframe::Result {
    let control_port = match control::port_from_args(std::env::args()) {
        Ok(port) => port,
        Err(e) => {
            eprintln!("keyprobe: {e}");
            std::process::exit(2);
        }
    };
    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let queue = KeyQueue::default();
    let app_queue = Arc::clone(&queue);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("KeyProbe")
            .with_inner_size([960.0, 420.0]),
        ..Default::default()
    };
    let inner = eframe::create_native(
        "KeyProbe",
        options,
        Box::new(move |cc| {
            let control = control_port.and_then(|port| Control::start(port, &cc.egui_ctx));
            Ok(Box::new(DesktopApp::new(app_queue, control)))
        }),
        &event_loop,
    );
    let mut tap = KeyTap { inner, queue };
    event_loop.run_app(&mut tap)?;
    Ok(())
}

/// Forwards every winit callback to eframe, copying key events into the queue on the way.
struct KeyTap<'a> {
    inner: eframe::EframeWinitApplication<'a>,
    queue: KeyQueue,
}

impl ApplicationHandler<UserEvent> for KeyTap<'_> {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let WindowEvent::KeyboardInput { event: key, .. } = &event {
            let down = key.state == ElementState::Pressed;
            if let Some(raw) =
                capture::raw_key(key.physical_key, &key.logical_key, down, key.repeat)
            {
                let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
                if q.len() >= MAX_QUEUED_KEYS {
                    q.pop_front();
                }
                q.push_back(raw);
            }
        }
        self.inner.window_event(event_loop, id, event);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.resumed(event_loop);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        self.inner.new_events(event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        self.inner.user_event(event_loop, event);
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, id: DeviceId, event: DeviceEvent) {
        self.inner.device_event(event_loop, id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.about_to_wait(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.suspended(event_loop);
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.exiting(event_loop);
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        self.inner.memory_warning(event_loop);
    }
}

struct DesktopApp {
    app: kp_engine::App,
    ui: KeyProbeUi,
    queue: KeyQueue,
    control: Option<Control>,
}

impl DesktopApp {
    fn new(queue: KeyQueue, control: Option<Control>) -> Self {
        Self {
            app: kp_engine::App::default(),
            ui: KeyProbeUi::default(),
            queue,
            control,
        }
    }

    fn drain_keys(&mut self) {
        let keys: Vec<RawKey> = {
            let mut q = self.queue.lock().unwrap_or_else(|e| e.into_inner());
            q.drain(..).collect()
        };
        for raw in keys {
            let (id, params) = raw.command();
            self.ui.run(&mut self.app, id, &params);
        }
    }
}

impl eframe::App for DesktopApp {
    /// Every key is input under test, so egui gets none: no Tab focus moves, no Space/Enter
    /// activating a focused button, no zoom shortcuts. The mouse still drives the UI.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        raw_input.events.retain(|e| {
            !matches!(
                e,
                egui::Event::Key { .. }
                    | egui::Event::Text(_)
                    | egui::Event::Copy
                    | egui::Event::Cut
                    | egui::Event::Paste(_)
                    | egui::Event::Ime(_)
            )
        });
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.drain_keys();
        if let Some(control) = &self.control {
            control.drain(&mut self.app, ui.ctx());
        }
        egui::CentralPanel::default().show(ui, |ui| self.ui.show(ui, &mut self.app));
    }
}
