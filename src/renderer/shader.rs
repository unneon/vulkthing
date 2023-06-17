use crate::renderer::util::Dev;
use ash::vk;
use log::{debug, error};
use shaderc::{CompilationArtifact, ResolvedInclude};
use std::ffi::CStr;

pub struct Shader<'a> {
    dev: &'a Dev,
    module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo,
}

pub struct SpecializationConstant {
    pub id: u32,
    pub value: i32,
}

impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe { self.dev.destroy_shader_module(self.module, None) };
    }
}

pub fn create_shader<'a>(
    glsl_path: &str,
    stage: vk::ShaderStageFlags,
    supports_raytracing: bool,
    specialization: &vk::SpecializationInfo,
    dev: &'a Dev,
) -> Shader<'a> {
    let code = compile_glsl(glsl_path, stage, supports_raytracing);
    let create_info = *vk::ShaderModuleCreateInfo::builder().code(code.as_binary());
    let module = unsafe { dev.create_shader_module(&create_info, None) }.unwrap();
    let stage_info = *vk::PipelineShaderStageCreateInfo::builder()
        .stage(stage)
        .module(module)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
        .specialization_info(specialization);
    Shader {
        dev,
        stage_info,
        module,
    }
}

fn compile_glsl(
    glsl_path: &str,
    stage: vk::ShaderStageFlags,
    supports_raytracing: bool,
) -> CompilationArtifact {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    if supports_raytracing {
        options.add_macro_definition("SUPPORTS_RAYTRACING", Some(""));
    }
    options.set_include_callback(|path, _, _, _| {
        Ok(ResolvedInclude {
            resolved_name: path.to_owned(),
            content: std::fs::read_to_string(format!("shaders/{}", path)).unwrap(),
        })
    });
    let glsl_text = std::fs::read_to_string(glsl_path).unwrap();
    let shader_kind = if stage.contains(vk::ShaderStageFlags::VERTEX) {
        shaderc::ShaderKind::Vertex
    } else if stage.contains(vk::ShaderStageFlags::FRAGMENT) {
        shaderc::ShaderKind::Fragment
    } else {
        panic!("unknown shader stage {stage:?}");
    };
    let compile_result =
        compiler.compile_into_spirv(&glsl_text, shader_kind, glsl_path, "main", Some(&options));
    let spirv_data = match compile_result {
        Err(shaderc::Error::CompilationError(_, output)) => {
            for message in output.trim().split('\n') {
                let (file, message) = message.split_once(':').unwrap();
                let (line, message) = message.split_once(':').unwrap();
                let message = message.strip_prefix(" error: '").unwrap();
                let (token, message) = message.split_once("' :").unwrap();
                let token_format = if token.is_empty() {
                    String::new()
                } else {
                    format!("{token}, ")
                };
                let message = message.trim();
                error!("shader compilation error, {token_format}{message}, \x1B[1mfile\x1B[0m: {file}, \x1B[1mline\x1B[0m: {line}");
            }
            panic!("shader compilation error");
        }
        result => result.unwrap(),
    };
    debug!("shader GLSL compiled, \x1B[1mfile\x1B[0m: {glsl_path}");
    spirv_data
}

pub fn create_specialization_entries(
    config: &[SpecializationConstant],
) -> Vec<vk::SpecializationMapEntry> {
    let mut entries = Vec::new();
    for (index, constant) in config.iter().enumerate() {
        entries.push(vk::SpecializationMapEntry {
            constant_id: constant.id,
            offset: (index * std::mem::size_of::<SpecializationConstant>() + 4) as u32,
            size: 4,
        });
    }
    entries
}

pub fn create_specialization(
    config: &[SpecializationConstant],
    entries: &[vk::SpecializationMapEntry],
) -> vk::SpecializationInfo {
    let data =
        unsafe { std::slice::from_raw_parts(config.as_ptr() as *const u8, config.len() * 8) };
    *vk::SpecializationInfo::builder()
        .map_entries(entries)
        .data(data)
}
