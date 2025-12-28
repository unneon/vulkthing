#![feature(iter_array_chunks)]

mod config;
mod generate;
mod helper;
mod reflect;
mod shaders;
mod types;

use crate::generate::generate_code;
use crate::reflect::reflect_shaders;
use crate::shaders::compile_shaders;

const IN_PATH: &str = "renderer.kdl";

fn main() {
    let text = std::fs::read_to_string(IN_PATH).unwrap();
    let renderer = knuffel::parse(IN_PATH, &text).unwrap();
    compile_shaders(&renderer);
    let reflection = reflect_shaders();
    generate_code(&renderer, &reflection);
    println!("cargo:rerun-if-changed={IN_PATH}");
    println!("cargo:rerun-if-changed=shaders");
}
