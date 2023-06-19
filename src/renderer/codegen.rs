// Code generated from renderer.kdl.

use crate::renderer::util::Dev;
use ash::vk;

pub struct Samplers {
    pub pixel: vk::Sampler,
}

struct Scratch {
    pixel_sampler_info: vk::SamplerCreateInfo,
}

static mut SCRATCH: Scratch = Scratch {
    pixel_sampler_info: vk::SamplerCreateInfo {
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
};

impl Samplers {
    pub fn cleanup(&self, dev: &Dev) {
        unsafe { dev.destroy_sampler(self.pixel, None) };
    }
}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {
    let pixel = unsafe { dev.create_sampler(&SCRATCH.pixel_sampler_info, None) }.unwrap();
    Samplers {
        pixel,
    }
}
