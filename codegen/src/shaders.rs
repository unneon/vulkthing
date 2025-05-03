use crate::config::Renderer;
use std::path::PathBuf;
use std::process::Command;

pub fn compile_shaders(renderer: &Renderer) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (shader_name, shader_type) in renderer.shaders() {
        let spirv_path = out_dir.join(format!("{shader_name}.{}.spv", shader_type.extension()));
        let entry_point = format!("{shader_name}_{}", shader_type.extension());
        let status = Command::new("slangc")
            .args([
                "shaders/main.slang",
                "-profile",
                "glsl_460",
                "-entry",
                entry_point.as_str(),
                "-target",
                "spirv",
                // TODO: Slangc should infer this based on mesh shader usage, report as bug.
                "-capability",
                "spirv_1_4",
                "-O3",
                "-o",
            ])
            .arg(spirv_path)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
