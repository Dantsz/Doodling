use crate::{brush::Rectangle, render_state::State};
use log::info;
use std::sync::{Arc, Mutex};
use web_sys::console::warn;
use web_sys::window;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::Window,
};
#[derive(Debug)]
pub enum Events {
    NewState(Arc<Mutex<State>>),
    Close,
}

use crate::utils;

use utils::{WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Clone)]
pub struct CanvasApp {
    // request_redraw: bool,
    // wait_cancelled: bool,
    // close_requested: bool,
    mouse_pressed: bool,
    mouse_position: (f32, f32),
    window: Option<Arc<Window>>,
    pub state: Option<Arc<Mutex<State>>>,
    event_loop: Arc<Mutex<EventLoopProxy<Events>>>,
}
impl CanvasApp {
    pub fn renderer(&self) -> Arc<Mutex<State>> {
        self.state.as_ref().unwrap().clone()
    }
    pub fn new(event_loop: Arc<Mutex<EventLoopProxy<Events>>>) -> Self {
        Self {
            // request_redraw: false,
            // wait_cancelled: false,
            // close_requested: false,
            mouse_pressed: false,
            mouse_position: (0.0, 0.0),
            window: None,
            state: None,
            event_loop,
        }
    }
}
impl ApplicationHandler<Events> for CanvasApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .with_decorations(false);

        #[cfg(target_arch = "wasm32")]
        {
            info!("Initializing canvas");
            use web_sys::wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document.get_element_by_id("wasm-example").unwrap();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }
        self.window = Some(Arc::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let new_state = Arc::new(Mutex::new(pollster::block_on(State::new(
                self.window.as_ref().unwrap().clone(),
            ))));
            self.event_loop
                .lock()
                .unwrap()
                .send_event(Events::NewState(new_state))
                .expect("Failed to send new state event");
        }

        #[cfg(target_arch = "wasm32")]
        {
            let window = self.window.as_ref().unwrap().clone();
            let event_loop = self.event_loop.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let new_state = Arc::new(Mutex::new(State::new(window).await));
                event_loop
                    .lock()
                    .unwrap()
                    .send_event(Events::NewState(new_state))
                    .expect("Failed to send new state event");
            });
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.state.is_none() {
            println!("State is none");
            return;
        }
        let mut rect = {
            let rendptr = self.renderer();
            let mut state = rendptr.lock().unwrap();
            Rectangle::new(&mut state, [0.0, 0.0, 10.0, 10.0])
        };
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {} //TODO: exit
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                ..
            } => {
                if let WindowEvent::MouseInput { state: pressed, .. } = event {
                    self.mouse_pressed = pressed == ElementState::Pressed;
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Right,
                ..
            } => {}

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::RedrawRequested => {
                // let window = self.window.as_ref().unwrap();
                let rendptr = self.renderer();
                let mut renderer = rendptr.lock().unwrap();

                if self.mouse_pressed {
                    let mut paint = renderer.begin_render();
                    rect.draw_to(
                        &mut renderer,
                        &mut paint,
                        [self.mouse_position.0, self.mouse_position.1],
                    );
                    renderer.end_render(paint);
                }

                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => {}
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => unimplemented!(),
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            WindowEvent::Resized(size) => {
                println!("Resized to {:?}", size);
            }
            _ => {}
        }
    }

    fn new_events(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Events) {
        match event {
            Events::Close => {}
            Events::NewState(state) => {
                self.state = Some(state);
                let rendptr = self.renderer();
                let mut renderer = rendptr.lock().unwrap();
                let mut clear_screen = renderer.begin_render();
                renderer.clear_screen(&mut clear_screen);
                renderer.end_render(clear_screen);
                let _ = renderer.render();
            }
        }
    }

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }
}
