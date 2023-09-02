#![allow(non_snake_case)]
use DoodlingCanvas::{create_window};

fn main() {
    let window = pollster::block_on(create_window());
    window.run_window_loop();
}