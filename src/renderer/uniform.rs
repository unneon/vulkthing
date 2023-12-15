use crate::interface::EnumInterface;
use nalgebra::{Matrix4, Vector2, Vector3};
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ComputeParams {
    pub count: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Transform {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Material {
    pub albedo: Vector3<f32>,
    pub metallic: f32,
    pub emit: Vector3<f32>,
    pub roughness: f32,
    pub ao: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Global {
    pub grass: GrassUniform,
    pub light: Light,
    pub settings: Settings,
    pub atmosphere: Atmosphere,
    pub gaussian: Gaussian,
    pub postprocessing: PostprocessUniform,
    pub camera: Camera,
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct GrassUniform {
    pub height_average: f32,
    pub height_max_variance: f32,
    pub width: f32,
    pub time: f32,
    pub sway_direction: Vector3<f32>,
    pub sway_frequency: f32,
    pub sway_amplitude: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Light {
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub position: Vector3<f32>,
}

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Settings {
    pub use_ray_tracing: bool,
    pub _pad0: [u8; 3],
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Atmosphere {
    pub enable: bool,
    pub _pad0: [u8; 3],
    pub scatter_point_count: u32,
    pub optical_depth_point_count: u32,
    pub density_falloff: f32,
    pub planet_position: Vector3<f32>,
    pub planet_radius: f32,
    pub sun_position: Vector3<f32>,
    pub scale: f32,
    pub wavelengths: Vector3<f32>,
    pub scattering_strength: f32,
    pub henyey_greenstein_g: f32,
}

#[repr(C, align(4))]
#[derive(Clone, Copy)]
pub struct Gaussian {
    pub threshold: f32,
    pub radius: i32,
    pub exponent_coefficient: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct PostprocessUniform {
    pub color_filter: Vector3<f32>,
    pub bloom_constant: f32,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub tonemapper: Tonemapper,
    pub gamma: f32,
}

#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Camera {
    pub view_matrix: Matrix4<f32>,
    pub projection_matrix: Matrix4<f32>,
    pub inverse_view_matrix: Matrix4<f32>,
    pub inverse_projection_matrix: Matrix4<f32>,
    pub resolution: Vector2<f32>,
    pub _pad0: [f32; 2],
    pub position: Vector3<f32>,
}

#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Tonemapper {
    RgbClamping = 0,
    TumblinRushmeier = 1,
    Schlick = 2,
    Ward = 3,
    Reinhard = 4,
    ReinhardExtended = 5,
    Hable = 6,
    Uchimura = 7,
    NarkowiczAces = 8,
    HillAces = 9,
}

impl EnumInterface for Tonemapper {
    const VALUES: &'static [Self] = &[
        Tonemapper::RgbClamping,
        Tonemapper::Reinhard,
        Tonemapper::NarkowiczAces,
        Tonemapper::HillAces,
    ];

    fn label(&self) -> Cow<str> {
        Cow::Borrowed(match self {
            Tonemapper::RgbClamping => "RGB Clamping",
            Tonemapper::TumblinRushmeier => "Tumblin Rushmeier",
            Tonemapper::Schlick => "Schlick",
            Tonemapper::Ward => "Ward",
            Tonemapper::Reinhard => "Reinhard",
            Tonemapper::ReinhardExtended => "Reinhard extended",
            Tonemapper::Hable => "Hable",
            Tonemapper::Uchimura => "Uchimura",
            Tonemapper::NarkowiczAces => "Narkowicz ACES",
            Tonemapper::HillAces => "Hill ACES",
        })
    }
}
