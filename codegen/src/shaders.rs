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
    let output = command
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
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let mut error_found = false;
    for [msg, ctx, highlight] in stderr.lines().array_chunks() {
        let severity = if msg.contains(": error ") {
            error_found = true;
            "error"
        } else if msg.contains(": warning ") {
            "warning"
        } else {
            unreachable!()
        };
        println!("cargo::{severity}={msg}");
        println!("cargo::{severity}={ctx}");
        println!("cargo::{severity}={highlight}");
    }
    if error_found {
        std::process::exit(0);
    }
    assert!(output.status.success());
}
