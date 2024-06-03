#![allow(non_snake_case)]
mod brush;
mod render_state;
pub mod utils;
pub mod winit_app;
use std::sync::{Arc, Mutex};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::png::PngEncoder, EncodableLayout};
use log::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit_app::{CanvasApp, Events};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WindowHandler {
    event_loop: Arc<Mutex<Option<EventLoop<Events>>>>,
    event_loop_proxy: Arc<Mutex<EventLoopProxy<Events>>>,
    app: CanvasApp,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WindowHandler {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn new(other: &WindowHandler) -> Self {
        Self {
            event_loop: other.event_loop.clone(),
            event_loop_proxy: other.event_loop_proxy.clone(),
            app: other.app.clone(),
        }
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn run_window_loop(mut self) {
        info!("Setup window loop");
        let event_loop = self.event_loop.lock().unwrap().take().unwrap();
        info!("Running loop");
        let _ = event_loop.run_app(&mut self.app);
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_canvas_capture(&self) -> String {
        if self.app.state.is_none() {
            return String::from("");
        }
        let rendptr = self.app.renderer();
        let img = pollster::block_on(rendptr.lock().unwrap().extract_framebuffer());
        let mut buffer = Vec::new();
        let encoder = PngEncoder::new(&mut buffer);
        img.write_with_encoder(encoder).unwrap();
        let frame = STANDARD.encode(buffer.as_bytes());
        frame
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn close(&self) {
        info!("Sending close event");
        self.event_loop_proxy
            .lock()
            .unwrap()
            .send_event(Events::Close)
            .expect("Failed to send close event");
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn create_window() -> WindowHandler {
    cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info);
    } else {
        env_logger::init();
    }
    }
    info!("Creating window");
    let event_loop = EventLoop::<Events>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    let proxy = event_loop.create_proxy();

    let event_loop_proxy = Arc::new(Mutex::new(proxy));
    WindowHandler {
        event_loop: Arc::new(Mutex::new(Some(event_loop))),
        event_loop_proxy: event_loop_proxy.clone(),
        app: CanvasApp::new(event_loop_proxy.clone()),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn _entry_point() {}
