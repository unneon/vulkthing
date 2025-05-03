#![feature(path_add_extension)]

mod config;
mod generate;
mod helper;
mod shaders;
mod types;

use crate::config::Renderer;
use crate::generate::generate_code;
use crate::shaders::compile_shaders;
use std::fs::File;

pub fn build_script(in_path: &str, out_path: &str) {
    let text = std::fs::read_to_string(in_path).unwrap();
    let renderer: Renderer = knuffel::parse(in_path, &text).unwrap();
    let out_file = File::create(out_path).unwrap();
    compile_shaders(&renderer);
    generate_code(in_path, &renderer, out_file);
    println!("cargo:rerun-if-changed={in_path}");
    println!("cargo:rerun-if-changed=shaders");
}
