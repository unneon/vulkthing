#![feature(const_cstr_methods)]

mod model;
mod renderer;
mod window;

use crate::model::load_model;
use crate::renderer::run_renderer;
use crate::window::create_window;

fn main() {
    let window = create_window();
    let model = load_model();
    run_renderer(window, model);
}
