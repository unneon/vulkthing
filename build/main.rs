#![feature(iter_array_chunks)]

mod config;
mod generate;
mod helper;
mod reflect;
mod shaders;
mod types;

use crate::generate::descriptors::generate_descriptors;
use crate::generate::gpu::generate_gpu;
use crate::generate::pipelines::generate_pipelines;
use crate::generate::samplers::generate_samplers;
use crate::reflect::reflect_shaders;
use crate::shaders::compile_shaders;
use std::path::PathBuf;

const IN_PATH: &str = "renderer.kdl";

fn main() {
    let text = std::fs::read_to_string(IN_PATH).unwrap();
    let renderer = knuffel::parse(IN_PATH, &text).unwrap();
    compile_shaders(&renderer);
    let reflection = reflect_shaders();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    generate_descriptors(&reflection, &out_dir);
    generate_gpu(&reflection, &out_dir);
    generate_pipelines(&renderer, &out_dir);
    generate_samplers(&renderer, &out_dir);
    println!("cargo:rerun-if-changed={IN_PATH}");
    println!("cargo:rerun-if-changed=shaders");
}
