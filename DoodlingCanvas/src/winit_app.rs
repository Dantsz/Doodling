use std::sync::{Arc, Mutex};

use crate::{brush::Rectangle, render_state::State};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

#[derive(Debug)]
pub enum Events {
    Close,
}

use crate::utils;

use utils::{WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Clone, Default)]
pub struct CanvasApp<'a> {
    // request_redraw: bool,
    // wait_cancelled: bool,
    // close_requested: bool,
    mouse_pressed: bool,
    mouse_position: (f32, f32),
    window: Option<Arc<Window>>,
    pub state: Option<Arc<Mutex<State<'a>>>>,
}
impl<'a> CanvasApp<'a> {
    pub fn renderer(&self) -> Arc<Mutex<State<'a>>> {
        self.state.as_ref().unwrap().clone()
    }
}
impl<'a> ApplicationHandler<Events> for CanvasApp<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attribues = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        self.window = Some(Arc::new(
            event_loop.create_window(window_attribues).unwrap(),
        ));
        self.state = Some(Arc::new(Mutex::new(pollster::block_on(State::new(
            self.window.as_ref().unwrap().clone(),
        )))));
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

        self.window.as_ref().unwrap().pre_present_notify();
        let rendptr = self.renderer();
        let mut renderer = rendptr.lock().unwrap();
        let mut clear_screen = renderer.begin_render();
        renderer.clear_screen(&mut clear_screen);
        renderer.end_render(clear_screen);
        let _ = renderer.render();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
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
                let window = self.window.as_ref().unwrap();
                window.pre_present_notify();
                let rendptr = self.renderer();
                let mut renderer = rendptr.lock().unwrap();

                if self.mouse_pressed {
                    self.mouse_pressed = false;
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

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Events) {}

    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let window = self.window.as_ref().unwrap();
        window.request_redraw();
    }
}
