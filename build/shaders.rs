use crate::pipelines::{Pipeline, Shader};
use std::path::Path;
use std::process::Command;

pub fn compile_shaders(pipelines: &[Pipeline], out_dir: &Path) {
    let shaders_dir = out_dir.join("shaders");
    std::fs::create_dir_all(&shaders_dir).unwrap();
    for pipeline in pipelines {
        for shader in pipeline.shaders() {
            println!();
            println!(
                "----- COMPILING SHADER {:?} --------------",
                shader.glsl_path()
            );
            compile_shader(&shader, &shaders_dir);
        }
    }
}

fn compile_shader(shader: &Shader, shaders_dir: &Path) {
    let name = shader.pipeline_name();
    let stage = shader.stage();
    let mut command = Command::new("glslangValidator");
    command.args([
        "--target-env",
        "vulkan1.3",
        "--glsl-version",
        "460",
        "shaders/limits.conf",
        "-S",
        shader.glslang_validator_stage(),
    ]);
    command.arg(shader.glsl_path());
    command.arg("-o");
    command.arg(shaders_dir.join(format!("{name}.{stage}.spv")));
    if !command.status().unwrap().success() {
        std::process::exit(1);
    }
}
