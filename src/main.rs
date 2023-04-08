#![feature(const_cstr_methods)]
#![feature(once_cell)]

mod logger;
mod model;
mod renderer;
mod window;

use crate::logger::initialize_logger;
use crate::model::load_model;
use crate::renderer::run_renderer;
use crate::window::create_window;

fn main() {
    initialize_logger();
    let window = create_window();
    let model = load_model();
    run_renderer(window, model);
}
