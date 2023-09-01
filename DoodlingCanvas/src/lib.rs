#![allow(non_snake_case)]
mod render_state;
mod brush;
pub mod utils;
use utils::{WINDOW_HEIGHT, WINDOW_WIDTH};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::PhysicalSize,
};

use crate::{render_state::State, brush::Rectangle};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
    } else {
        env_logger::init();
    }
    }
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut mouse_position = (0.0, 0.0);
    let mut mouse_pressed : ElementState = ElementState::Released;
    window.set_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
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
    let state_window_id = window.id();
    let mut state = State::new(window).await;
    {
        let mut clear_screen = state.begin_render();
        state.clear_screen(&mut clear_screen);
        state.end_render(clear_screen);
    }
    //state.present();

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
            } => {pollster::block_on(state.extract_framebuffer()); *control_flow = ControlFlow::Exit},
            WindowEvent::MouseInput{
                button:MouseButton::Left,
                ..
            } => {
                let WindowEvent::MouseInput{state : pressed,..} = *event else{unimplemented!()};
                mouse_pressed = pressed;
            },
            WindowEvent::CursorMoved{position,..} => {
                mouse_position = (position.x as f32, position.y as f32);
            },
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            if mouse_pressed == ElementState::Pressed {
                let mut paint = state.begin_render();
                let mut rect = Rectangle::new([mouse_position.0,mouse_position.1,10.0,10.0]);
                rect.draw_to(&mut state, &mut paint);
                state.end_render(paint);
            }
            match state.render() {
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
            state.window().request_redraw();
        },
        _ => {}
    }
    });

}