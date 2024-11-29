use crate::config::Renderer;
use std::path::PathBuf;
use std::process::Command;

pub fn compile_shaders(renderer: &Renderer) {
    for (shader_name, shader_type) in renderer.shaders() {
        let glsl_path = PathBuf::from(format!("shaders/{shader_name}.{}", shader_type.extension()));
        if !glsl_path.exists() {
            let spirv_path = glsl_path.with_added_extension("spv");
            let entry_point = format!("{shader_name}_{}", shader_type.extension());
            Command::new("slangc")
                .args([
                    "shaders/main.slang",
                    "-profile",
                    "glsl_460",
                    "-entry",
                    entry_point.as_str(),
                    "-target",
                    "spirv",
                    "-o",
                ])
                .arg(spirv_path)
                .status()
                .unwrap();
        }
    }
}
