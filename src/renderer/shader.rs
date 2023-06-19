use crate::renderer::util::Dev;
use ash::vk;
use log::{debug, error};
use shaderc::{CompilationArtifact, ResolvedInclude};

pub struct Shader<'a> {
    dev: &'a Dev,
    pub module: vk::ShaderModule,
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
    dev: &'a Dev,
) -> Shader<'a> {
    let code = compile_glsl(glsl_path, stage, supports_raytracing);
    let create_info = *vk::ShaderModuleCreateInfo::builder().code(code.as_binary());
    let module = unsafe { dev.create_shader_module(&create_info, None) }.unwrap();
    Shader { dev, module }
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
