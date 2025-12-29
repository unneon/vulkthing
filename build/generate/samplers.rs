use crate::config::Renderer;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn generate_samplers(renderer: &Renderer, out_dir: &Path) {
    let mut file = File::create(out_dir.join("samplers.rs")).unwrap();

    write!(file, r"pub struct Samplers {{").unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    pub {sampler}: vk::Sampler,").unwrap();
    }
    writeln!(file, "}}").unwrap();

    for sampler in &renderer.samplers {
        let filter = &sampler.filter;
        let address_mode = &sampler.address_mode;
        let unnormalized_coordinates = sampler.unnormalized_coordinates as u32;
        writeln!(
            file,
            r#"
static SAMPLER_{sampler}: vk::SamplerCreateInfo = vk::SamplerCreateInfo {{
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::{filter},
        min_filter: vk::Filter::{filter},
        mipmap_mode: vk::SamplerMipmapMode::NEAREST,
        address_mode_u: vk::SamplerAddressMode::{address_mode},
        address_mode_v: vk::SamplerAddressMode::{address_mode},
        address_mode_w: vk::SamplerAddressMode::{address_mode},
        mip_lod_bias: 0.,
        anisotropy_enable: 0,
        max_anisotropy: 0.,
        compare_enable: 0,
        compare_op: vk::CompareOp::NEVER,
        min_lod: 0.,
        max_lod: 0.,
        border_color: vk::BorderColor::FLOAT_TRANSPARENT_BLACK,
        unnormalized_coordinates: {unnormalized_coordinates},
    }};"#
        )
        .unwrap();
    }

    writeln!(
        file,
        r#"
pub fn create_samplers(dev: &Dev) -> Samplers {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "    let {sampler} = unsafe {{ dev.create_sampler(&SAMPLER_{sampler}, None).unwrap_unchecked() }};").unwrap();
    }
    writeln!(file, "    Samplers {{").unwrap();
    for sampler in &renderer.samplers {
        writeln!(file, "        {},", sampler.name).unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}

impl Samplers {{
    pub fn cleanup(&self, dev: &Dev) {{"#
    )
    .unwrap();
    for sampler in &renderer.samplers {
        writeln!(
            file,
            "        unsafe {{ dev.destroy_sampler(self.{sampler}, None) }};"
        )
        .unwrap();
    }
    writeln!(
        file,
        r#"    }}
}}"#
    )
    .unwrap();
}
