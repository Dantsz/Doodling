use crate::{brush::Rectangle, render_state::State, utils};
use log::info;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
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
//maybe should just return
pub type GetFramebufferAction =
    Arc<Mutex<Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = image::RgbaImage>>>>>>>; //Look at this! This comment was made before adding Pin :(

#[allow(dead_code)]
pub struct CanvasApp {
    mouse_pressed: bool,
    mouse_position: (f32, f32),
    window: Option<Arc<Window>>,
    pub state: Option<Arc<Mutex<State>>>,
    event_loop: Arc<Mutex<EventLoopProxy<Events>>>,
    get_framebuffer: GetFramebufferAction,
}

impl CanvasApp {
    pub fn renderer(&self) -> Arc<Mutex<State>> {
        self.state.as_ref().unwrap().clone()
    }
    pub fn new(
        event_loop: Arc<Mutex<EventLoopProxy<Events>>>,
        get_framebuffer: GetFramebufferAction,
    ) -> Self {
        Self {
            mouse_pressed: false,
            mouse_position: (0.0, 0.0),
            window: None,
            state: None,
            event_loop,
            get_framebuffer,
        }
    }
}
impl ApplicationHandler<Events> for CanvasApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes()
            .with_max_inner_size(LogicalSize::new(utils::WINDOW_WIDTH, utils::WINDOW_HEIGHT));

        #[cfg(target_arch = "wasm32")]
        {
            info!("Initializing canvas");
            use wasm_bindgen::JsCast;
            use web_sys::HtmlCanvasElement;
            use winit::platform::web::WindowAttributesExtWebSys;
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            let canvas = document.get_element_by_id("canvas").unwrap();
            let html_canvas_element: HtmlCanvasElement = {
                match canvas.dyn_into() {
                    Ok(val) => val,
                    Err(_) => {
                        log::warn!("Failed to convert canvas to html canvas element");
                        return;
                    }
                }
            };
            log::info!("Canvas created ({})", html_canvas_element.outer_html());

            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }
        let window = event_loop
            .create_window(window_attributes)
            .expect("Failed to create window");

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas().expect("Failed to get canvas");
            log::info!("Window canvas size: ({})", canvas.outer_html());
        }
        self.window = Some(Arc::new(window));
        log::info!(
            "Window created {:?}",
            self.window.as_ref().unwrap().inner_size()
        );
        #[cfg(not(target_arch = "wasm32"))]
        {
            let new_state = Arc::new(Mutex::new(pollster::block_on(State::new(
                self.window.as_ref().unwrap().clone(),
            ))));
            log::warn!("GetFramebuffer not implemented for non-wasm32");
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
            let get_framebuffer = self.get_framebuffer.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let new_state = Arc::new(Mutex::new(State::new(window).await));
                let fb_state = new_state.clone();
                log::info!("Setting canvas capture");
                get_framebuffer.lock().unwrap().replace(Box::new(move || {
                    let fb_state = fb_state.clone();
                    Box::pin(async move {
                        let fb_state = fb_state.clone();
                        let fb_state = fb_state.lock().unwrap();
                        fb_state.extract_framebuffer().await
                    })
                }));

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
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if self.state.is_none() {
            log::warn!("Cannot process window events: state is none");
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
            } => {
                #[cfg(not(target_arch = "wasm32"))]
                event_loop.exit();
            }

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
            WindowEvent::Resized(new_size) => {
                println!("Resized to {:?}", new_size);
                if let Some(state) = self.state.as_ref() {
                    if new_size.width > 0 && new_size.height > 0 {
                        let mut state_lock = state.lock().unwrap();
                        state_lock.config.width = new_size.width;
                        state_lock.config.height = new_size.height;
                        state_lock
                            .surface
                            .configure(&state_lock.device, &state_lock.config);
                    }
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

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: Events) {
        info!("User event: {:?}", event);
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
