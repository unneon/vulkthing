use crate::renderer::util::exists_newer_file;
use ash::{vk, Device};
use log::{debug, error};
use std::ffi::CStr;
use std::fs::File;

pub struct Shader<'a> {
    logical_device: &'a Device,
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo,
}

pub struct SpecializationConstant {
    pub id: u32,
    pub value: i32,
}

impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe { self.logical_device.destroy_shader_module(self.module, None) };
    }
}

pub fn create_shader<'a>(
    glsl_path: &str,
    stage: vk::ShaderStageFlags,
    supports_raytracing: bool,
    specialization_info: &vk::SpecializationInfo,
    logical_device: &'a Device,
) -> Shader<'a> {
    let spirv_path = format!("{glsl_path}.spv");
    if !exists_newer_file(&spirv_path, glsl_path) {
        compile_shader(glsl_path, &spirv_path, stage, supports_raytracing);
    }
    load_shader(logical_device, &spirv_path, stage, specialization_info)
}

fn compile_shader(
    glsl_path: &str,
    spirv_path: &str,
    stage: vk::ShaderStageFlags,
    supports_raytracing: bool,
) {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    if supports_raytracing {
        options.add_macro_definition("SUPPORTS_RAYTRACING", Some(""));
    }
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
    std::fs::write(spirv_path, spirv_data.as_binary_u8()).unwrap();
    debug!("shader GLSL compiled, \x1B[1mfile\x1B[0m: {glsl_path}");
}

fn load_shader<'a>(
    logical_device: &'a Device,
    spirv_path: &str,
    stage: vk::ShaderStageFlags,
    specialization_info: &vk::SpecializationInfo,
) -> Shader<'a> {
    let mut file = File::open(spirv_path).unwrap();
    let aligned_code = ash::util::read_spv(&mut file).unwrap();
    let module = unsafe {
        logical_device.create_shader_module(
            &vk::ShaderModuleCreateInfo::builder().code(&aligned_code),
            None,
        )
    }
    .unwrap();
    debug!("shader SPIR-V loaded, \x1B[1mfile\x1B[0m: {spirv_path}");
    let stage_info = *vk::PipelineShaderStageCreateInfo::builder()
        .stage(stage)
        .module(module)
        .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
        .specialization_info(specialization_info);
    Shader {
        logical_device,
        stage_info,
        module,
    }
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
