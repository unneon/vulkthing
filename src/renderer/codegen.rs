// Code generated from renderer.kdl.

use crate::renderer::util::Dev;
use ash::vk;

pub struct Samplers {
    pub pixel: vk::Sampler,
}

pub struct DescriptorSetLayouts {
    pub object: vk::DescriptorSetLayout,
    pub grass: vk::DescriptorSetLayout,
    pub skybox: vk::DescriptorSetLayout,
    pub atmosphere: vk::DescriptorSetLayout,
    pub gaussian: vk::DescriptorSetLayout,
    pub postprocess: vk::DescriptorSetLayout,
}

struct Scratch {
    pixel_sampler: vk::SamplerCreateInfo,
    object_bindings: [vk::DescriptorSetLayoutBinding; 5],
    object_layout: vk::DescriptorSetLayoutCreateInfo,
    grass_bindings: [vk::DescriptorSetLayoutBinding; 5],
    grass_layout: vk::DescriptorSetLayoutCreateInfo,
    skybox_bindings: [vk::DescriptorSetLayoutBinding; 1],
    skybox_layout: vk::DescriptorSetLayoutCreateInfo,
    atmosphere_bindings: [vk::DescriptorSetLayoutBinding; 4],
    atmosphere_layout: vk::DescriptorSetLayoutCreateInfo,
    gaussian_bindings: [vk::DescriptorSetLayoutBinding; 2],
    gaussian_layout: vk::DescriptorSetLayoutCreateInfo,
    postprocess_bindings: [vk::DescriptorSetLayoutBinding; 3],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo,
}

#[rustfmt::skip]
static mut SCRATCH: Scratch = Scratch {
    pixel_sampler: vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_v: vk::SamplerAddressMode::CLAMP_TO_BORDER,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: 1,
    },
    object_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 4,
            descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    object_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 5,
        p_bindings: unsafe { SCRATCH.object_bindings.as_ptr() },
    },
    grass_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 4,
            descriptor_type: vk::DescriptorType::ACCELERATION_STRUCTURE_KHR,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    grass_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 5,
        p_bindings: unsafe { SCRATCH.grass_bindings.as_ptr() },
    },
    skybox_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    skybox_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 1,
        p_bindings: unsafe { SCRATCH.skybox_bindings.as_ptr() },
    },
    atmosphere_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::INPUT_ATTACHMENT,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 3,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    atmosphere_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 4,
        p_bindings: unsafe { SCRATCH.atmosphere_bindings.as_ptr() },
    },
    gaussian_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    gaussian_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 2,
        p_bindings: unsafe { SCRATCH.gaussian_bindings.as_ptr() },
    },
    postprocess_bindings: [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
        vk::DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            stage_flags: vk::ShaderStageFlags::FRAGMENT,
            p_immutable_samplers: std::ptr::null(),
        },
    ],
    postprocess_layout: vk::DescriptorSetLayoutCreateInfo {
        s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DescriptorSetLayoutCreateFlags::empty(),
        binding_count: 3,
        p_bindings: unsafe { SCRATCH.postprocess_bindings.as_ptr() },
    },
};

impl Samplers {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_sampler(self.pixel, None) };
    }
}

impl DescriptorSetLayouts {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_descriptor_set_layout(self.object, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.grass, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.skybox, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.atmosphere, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.gaussian, None) };
        unsafe { dev.destroy_descriptor_set_layout(self.postprocess, None) };
    }
}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {
    let pixel = unsafe { dev.create_sampler(&SCRATCH.pixel_sampler, None).unwrap_unchecked() };
    Samplers {
        pixel,
    }
}

#[rustfmt::skip]
pub fn create_descriptor_set_layouts(samplers: &Samplers, dev: &Dev) -> DescriptorSetLayouts {
    unsafe { SCRATCH.gaussian_bindings[0].p_immutable_samplers = &samplers.pixel };
    unsafe { SCRATCH.postprocess_bindings[0].p_immutable_samplers = &samplers.pixel };
    unsafe { SCRATCH.postprocess_bindings[1].p_immutable_samplers = &samplers.pixel };
    let object = unsafe { dev.create_descriptor_set_layout(&SCRATCH.object_layout, None).unwrap_unchecked() };
    let grass = unsafe { dev.create_descriptor_set_layout(&SCRATCH.grass_layout, None).unwrap_unchecked() };
    let skybox = unsafe { dev.create_descriptor_set_layout(&SCRATCH.skybox_layout, None).unwrap_unchecked() };
    let atmosphere = unsafe { dev.create_descriptor_set_layout(&SCRATCH.atmosphere_layout, None).unwrap_unchecked() };
    let gaussian = unsafe { dev.create_descriptor_set_layout(&SCRATCH.gaussian_layout, None).unwrap_unchecked() };
    let postprocess = unsafe { dev.create_descriptor_set_layout(&SCRATCH.postprocess_layout, None).unwrap_unchecked() };
    DescriptorSetLayouts {
        object,
        grass,
        skybox,
        atmosphere,
        gaussian,
        postprocess,
    }
}
