//! wasm32 only: page start-up, key listeners and the eframe app.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use eframe::egui;
use kp_ui::KeyProbeUi;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{AddEventListenerOptions, HtmlCanvasElement, KeyboardEvent, Window};

use crate::keys::{self, WebInput};

/// Input not yet turned into commands; bounded so a stalled page can't grow it forever.
const MAX_QUEUED: usize = 1024;

type Queue = Rc<RefCell<VecDeque<WebInput>>>;
/// The egui context, once eframe has created it; listeners use it to request a repaint.
type Waker = Rc<RefCell<Option<egui::Context>>>;

#[wasm_bindgen(start)]
pub fn start() {
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = run().await {
            web_sys::console::error_2(&"keyprobe: start failed:".into(), &e);
        }
    });
}

async fn run() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let canvas = window
        .document()
        .ok_or("no document")?
        .get_element_by_id("keyprobe")
        .ok_or("no <canvas id=\"keyprobe\">")?
        .dyn_into::<HtmlCanvasElement>()?;

    let queue = Queue::default();
    let waker = Waker::default();
    listen_keys(&window, "keydown", true, &queue, &waker)?;
    listen_keys(&window, "keyup", false, &queue, &waker)?;
    listen_blur(&window, &queue, &waker)?;

    let app_queue = Rc::clone(&queue);
    eframe::WebRunner::new()
        .start(
            canvas,
            eframe::WebOptions::default(),
            Box::new(move |cc| {
                *waker.borrow_mut() = Some(cc.egui_ctx.clone());
                Ok(Box::new(WebApp {
                    app: kp_engine::App::default(),
                    ui: KeyProbeUi::default(),
                    queue: app_queue,
                }))
            }),
        )
        .await
}

fn push(queue: &Queue, waker: &Waker, input: WebInput) {
    {
        let mut q = queue.borrow_mut();
        if q.len() >= MAX_QUEUED {
            q.pop_front();
        }
        q.push_back(input);
    }
    if let Some(ctx) = waker.borrow().as_ref() {
        ctx.request_repaint();
    }
}

fn listen_keys(
    window: &Window,
    kind: &str,
    down: bool,
    queue: &Queue,
    waker: &Waker,
) -> Result<(), JsValue> {
    let (queue, waker) = (Rc::clone(queue), Rc::clone(waker));
    let callback = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
        // Every key is input under test: stop the browser acting on it (Tab focus, Space
        // scroll, F5 reload, Alt menu…) where it lets us; some shortcuts it never hands over.
        e.prevent_default();
        if let Some(input) = keys::key_input(
            &e.code(),
            &e.key(),
            e.key_code(),
            down,
            e.repeat(),
            e.time_stamp(),
        ) {
            push(&queue, &waker, input);
        }
    });
    let options = AddEventListenerOptions::new();
    options.set_capture(true);
    window.add_event_listener_with_callback_and_add_event_listener_options(
        kind,
        callback.as_ref().unchecked_ref(),
        &options,
    )?;
    // The listener lives as long as the page.
    callback.forget();
    Ok(())
}

fn listen_blur(window: &Window, queue: &Queue, waker: &Waker) -> Result<(), JsValue> {
    let (queue, waker) = (Rc::clone(queue), Rc::clone(waker));
    let callback = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        push(&queue, &waker, WebInput::ReleaseAll);
    });
    window.add_event_listener_with_callback("blur", callback.as_ref().unchecked_ref())?;
    callback.forget();
    Ok(())
}

struct WebApp {
    app: kp_engine::App,
    ui: KeyProbeUi,
    queue: Queue,
}

impl eframe::App for WebApp {
    /// Same as desktop: egui gets no keys, so none can trigger UI actions.
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
        let input: Vec<WebInput> = self.queue.borrow_mut().drain(..).collect();
        for i in input {
            let (id, params) = i.command();
            self.ui.run(&mut self.app, id, &params);
        }
        egui::CentralPanel::default().show(ui, |ui| self.ui.show(ui, &mut self.app));
    }
}
