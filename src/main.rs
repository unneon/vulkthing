#![feature(const_cstr_methods)]
#![feature(const_option)]
#![feature(const_result_drop)]
#![feature(once_cell)]

mod camera;
mod input;
mod logger;
mod model;
mod renderer;
mod window;

use crate::logger::initialize_logger;
use crate::model::load_model;
use crate::renderer::run_renderer;
use crate::window::create_window;

const MOVEMENT_SPEED: f32 = 2.;
const CAMERA_SENSITIVITY: f32 = 0.01;

fn main() {
    initialize_logger();
    let window = create_window();
    let model = load_model();
    run_renderer(window, model);
}
