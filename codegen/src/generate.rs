use crate::config::Renderer;
use std::fs::File;
use std::io::Write;

pub fn generate_code(in_path: &str, renderer: &Renderer, mut file: File) {
    writeln!(
        file,
        r#"// Code generated from {in_path}.

use crate::renderer::util::Dev;
use ash::vk;

pub struct Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    pub {}: vk::Sampler,", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"}}

struct Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            "    {}_sampler_info: vk::SamplerCreateInfo,",
            sampler.name
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"}}

static mut SCRATCH: Scratch = Scratch {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            r"    {}_sampler_info: vk::SamplerCreateInfo {{
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::NEAREST,
        min_filter: vk::Filter::NEAREST,
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::{},
        address_mode_v: vk::SamplerAddressMode::{},
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: {},
    }},",
            sampler.name,
            sampler.address_mode_u,
            sampler.address_mode_v,
            if sampler.unnormalized_coordinates {
                1
            } else {
                0
            }
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"}};

impl Samplers {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_sampler(self.{}, None) }};",
            sampler.name
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

#[rustfmt::skip]
pub fn create_samplers(dev: &Dev) -> Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    let {} = unsafe {{ dev.create_sampler(&SCRATCH.{}_sampler_info, None) }}.unwrap();", sampler.name, sampler.name).unwrap();
    }
    writeln!(file, "    Samplers {{").unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "        {},", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}"#
    )
    .unwrap();
}
