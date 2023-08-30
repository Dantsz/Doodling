#![allow(non_snake_case)]
use DoodlingCanvas::run;

fn main() {
    pollster::block_on(run());
}
