#![allow(non_snake_case)]
mod render_state;
mod brush;
pub mod utils;
use std::sync::{Arc, Mutex};

use image::{codecs::png::PngEncoder, EncodableLayout};
use log::{info, error};
use utils::{WINDOW_HEIGHT, WINDOW_WIDTH};
use base64::{engine::general_purpose::STANDARD, Engine};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console::info;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder},
    window::{WindowBuilder, Window}, dpi::PhysicalSize,
};

use crate::{render_state::State, brush::Rectangle};
#[derive(Debug)]
enum Events{
    Close
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WindowHandler
{
    renderer: Arc<Mutex<State>>,
    event_loop: Arc<Mutex<Option<EventLoop<Events>>>>,
    event_loop_proxy: Arc<Mutex<EventLoopProxy<Events>>>
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WindowHandler
{
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn new(other : &WindowHandler) -> Self
    {
        Self
        {
            renderer: other.renderer.clone(),
            event_loop: other.event_loop.clone(),
            event_loop_proxy: other.event_loop_proxy.clone()
        }
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn run_window_loop(self)
    {
        info!("Setup window loop");
        let event_loop = self.event_loop.lock().unwrap().take().unwrap();
        let mut mouse_position = (0.0, 0.0);
        let mut mouse_pressed : ElementState = ElementState::Released;
        let state_window_id = {self.renderer.lock().unwrap().window().id()};
        {
            let mut renderer = self.renderer.lock().unwrap();
            let mut clear_screen = renderer.begin_render();
            renderer.clear_screen(&mut clear_screen);
            renderer.end_render(clear_screen);
        }
        info!("Running loop");

        let mut rect = {
        let mut state = self.renderer.lock().unwrap();
        Rectangle::new(&mut state,[0.0,0.0,10.0,10.0])
        };
        event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state_window_id => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => {*control_flow = ControlFlow::Exit},
                WindowEvent::MouseInput{
                    button:MouseButton::Left,
                    ..
                } => {
                    let WindowEvent::MouseInput{state : pressed,..} = *event else{unimplemented!()};
                    mouse_pressed = pressed;
                },
                WindowEvent::MouseInput {state: ElementState::Pressed, button: MouseButton::Right,..} =>
                {
                }

                WindowEvent::CursorMoved{position,..} => {
                    mouse_position = (position.x as f32, position.y as f32);
                },
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == state_window_id => {
                let mut renderer = self.renderer.lock().unwrap();
                if mouse_pressed == ElementState::Pressed {
                    let mut paint = renderer.begin_render();
                    rect.draw_to( &mut renderer, &mut paint,[mouse_position.0,mouse_position.1]);
                    renderer.end_render(paint);
                }
                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {},
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            },
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                let renderer = self.renderer.lock().unwrap();
                renderer.window().request_redraw();
            },
            Event::UserEvent(Events::Close) => {
                info!("Received close event,sending exit signal");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
        });
    }
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_canvas_capture(&self) -> String
    {
        let img = pollster::block_on(self.renderer.lock().unwrap().extract_framebuffer());
        let mut buffer = Vec::new();
        let encoder = PngEncoder::new(&mut buffer);
        img.write_with_encoder(encoder).unwrap();
        let frame = STANDARD.encode(buffer.as_bytes());
        frame
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn close(&self)
    {
        info!("Sending close event");
        self.event_loop_proxy.lock().unwrap().send_event(Events::Close).expect("Failed to send close event");
    }

}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn create_window() -> WindowHandler {
    cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info);
    } else {
        env_logger::init();
    }
    }
    info!("Creating window");
    let event_loop = EventLoopBuilder::<Events>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window: Window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        info!("Initializing canvas");
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    let state = Arc::new(Mutex::new(State::new(window).await));
    WindowHandler
    {
        renderer: state,
        event_loop : Arc::new(Mutex::new(Some(event_loop))),
        event_loop_proxy: Arc::new(Mutex::new(proxy))
    }

}



#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn _entry_point()
{

}