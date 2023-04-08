use crate::renderer::VulkanLogicalDevice;
use ash::vk;
use std::ffi::CStr;

pub struct Shader<'a> {
    logical_device: &'a VulkanLogicalDevice<'a>,
    pub module: vk::ShaderModule,
    pub stage_info: vk::PipelineShaderStageCreateInfo,
}

impl<'a> Shader<'a> {
    pub(super) fn compile(
        logical_device: &'a VulkanLogicalDevice,
        code: &'static [u8],
        stage: vk::ShaderStageFlags,
    ) -> Self {
        // Shaders need to be passed to vulkan as an aligned u32 array. It would be good to read
        // this from a file later, but memory mapping probably doesn't make sense for it? Even
        // simple shaders reach into the 2KB range though, so maybe.
        let aligned_code = ash::util::read_spv(&mut std::io::Cursor::new(code)).unwrap();
        let module = unsafe {
            logical_device.device.create_shader_module(
                &vk::ShaderModuleCreateInfo::builder().code(&aligned_code),
                None,
            )
        }
        .unwrap();
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
