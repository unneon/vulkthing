#![feature(iter_array_chunks)]

mod generate;
mod pipelines;
mod reflect;
mod shaders;
mod types;

use crate::generate::descriptors::generate_descriptors;
use crate::generate::gpu::generate_gpu;
use crate::generate::pipelines::generate_pipelines;
use crate::generate::samplers::generate_samplers;
use crate::pipelines::collect_pipelines;
use crate::reflect::reflect_shaders;
use crate::shaders::compile_shaders;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let pipelines = collect_pipelines();
    compile_shaders(&pipelines, &out_dir);
    let reflection = reflect_shaders();
    generate_descriptors(&reflection, &out_dir);
    generate_gpu(&reflection, &out_dir);
    generate_pipelines(&pipelines, &out_dir);
    generate_samplers(&out_dir);
    println!("cargo:rerun-if-changed=shaders");
}
