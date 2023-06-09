use crate::interface::EnumInterface;
use nalgebra::{Matrix4, Vector3};
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModelViewProjection {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Material {
    pub diffuse: Vector3<f32>,
    pub _pad0: f32,
    pub emit: Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Light {
    pub color: Vector3<f32>,
    pub ambient_strength: f32,
    pub position: Vector3<f32>,
    pub diffuse_strength: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FragSettings {
    pub use_ray_tracing: bool,
    pub _pad0: [u8; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Postprocessing {
    pub color_filter: Vector3<f32>,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub tonemapper: Tonemapper,
    pub gamma: f32,
    pub atmosphere: bool,
    pub _pad0: [u8; 3],
    pub atmosphere_scatter_point_count: u32,
    pub atmosphere_optical_depth_point_count: u32,
    pub atmosphere_density_falloff: f32,
    pub atmosphere_radius: f32,
    pub planet_radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Camera {
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
