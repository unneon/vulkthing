use ash::{vk, Device};
use log::debug;
use std::ffi::CStr;
use std::fs::File;

pub struct Shader<'a> {
    logical_device: &'a Device,
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo,
}

impl<'a> Shader<'a> {
    pub(super) fn compile(
        logical_device: &'a Device,
        spirv_path: &str,
        stage: vk::ShaderStageFlags,
    ) -> Self {
        let mut file = File::open(spirv_path).unwrap();
        let aligned_code = ash::util::read_spv(&mut file).unwrap();
        let module = unsafe {
            logical_device.create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(&aligned_code),
                None,
            )
        }
        .unwrap();
        debug!("shader SPIR-V loaded, \x1B[1mpath\x1B[0m: {spirv_path}");
        let stage_info = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .module(module)
            .name(CStr::from_bytes_with_nul(b"main\0").unwrap())
            .build();
        Shader {
            logical_device,
            stage_info,
            module,
        }
    }
}

impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe { self.logical_device.destroy_shader_module(self.module, None) };
    }
}
