#![allow(non_snake_case)]
use DoodlingCanvas::{create_window, run_window_loop};

fn main() {
    let window = pollster::block_on(create_window());
    run_window_loop(window);
}