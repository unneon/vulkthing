#![feature(const_cstr_methods)]

mod model;
mod renderer;

use crate::model::load_model;
use crate::renderer::run_renderer;

fn main() {
    let model = load_model();
    run_renderer(&model);
}
