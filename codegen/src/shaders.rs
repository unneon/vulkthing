use crate::config::Renderer;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn compile_shaders(renderer: &Renderer) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    compile_shader(None, &out_dir.join("reflection.spv"));
    let shaders_dir = out_dir.join("shaders");
    std::fs::create_dir_all(&shaders_dir).unwrap();
    for (shader_name, shader_type) in renderer.shaders() {
        let spirv_path = shaders_dir.join(format!("{shader_name}.{}.spv", shader_type.extension()));
        let entry_point = format!("{shader_name}_{}", shader_type.extension());
        compile_shader(Some(&entry_point), &spirv_path);
    }
}

fn compile_shader(entry_point: Option<&str>, spirv_path: &Path) {
    let mut command = Command::new("slangc");
    command.args(["shaders/main.slang", "-profile", "glsl_460"]);
    if let Some(entry_point) = entry_point {
        command.args(["-entry", entry_point]);
    }
    let status = command
        .args([
            "-target",
            "spirv",
            // TODO: Slangc should infer this based on mesh shader usage, report as bug.
            "-capability",
            "spirv_1_4",
            "-O3",
            "-o",
        ])
        .arg(spirv_path)
        // TODO: Parse warnings and errors and emit them to stdout in Cargo format.
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
}
