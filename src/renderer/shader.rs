use log::{debug, error};
use shaderc::{EnvVersion, Limit, ResolvedInclude, ShaderKind, TargetEnv};

pub fn compile_glsl(glsl_path: &str, shader_kind: ShaderKind) -> Vec<u32> {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_3 as u32);
    options.set_include_callback(|path, _, _, _| {
        Ok(ResolvedInclude {
            resolved_name: path.to_owned(),
            content: std::fs::read_to_string(format!("shaders/{}", path)).unwrap(),
        })
    });
    options.set_limit(Limit::MaxMeshWorkGroupSizeXExt, 256);
    let glsl_text = std::fs::read_to_string(glsl_path).unwrap();
    let compile_result =
        compiler.compile_into_spirv(&glsl_text, shader_kind, glsl_path, "main", Some(&options));
    let spirv_data = match compile_result {
        Err(shaderc::Error::CompilationError(_, output)) => {
            // TODO: Some error messages (debugPrintfEXT w/o extension decl) are multi-line.
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
            std::process::exit(1);
        }
        result => result.unwrap(),
    };
    debug!("shader GLSL compiled, \x1B[1mfile\x1B[0m: {glsl_path}");
    spirv_data.as_binary().to_owned()
}
