#![feature(array_chunks)]
#![feature(const_cstr_methods)]
#![feature(const_option)]
#![feature(const_result_drop)]

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
use ash::vk;
use std::ffi::CStr;

const VULKAN_APP_NAME: &CStr = CStr::from_bytes_with_nul(b"Vulkthing\0").ok().unwrap();
const VULKAN_APP_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);
const VULKAN_ENGINE_NAME: &CStr = CStr::from_bytes_with_nul(b"Vulkthing\0").ok().unwrap();
const VULKAN_ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 0, 0);

const MOVEMENT_SPEED: f32 = 2.;
const CAMERA_SENSITIVITY: f32 = 0.01;

fn main() {
    initialize_logger();
    let window = create_window();
    let model = load_model();
    run_renderer(window, model);
}
