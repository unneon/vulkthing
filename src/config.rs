use crate::planet::{NoiseType, Planet};
use crate::renderer::uniform::{FragSettings, Postprocessing, Tonemapper};
use nalgebra::Vector3;

pub const DEFAULT_PLANET: Planet = Planet {
    resolution: 400,
    radius: 100.,
    noise_type: NoiseType::Ridge,
    noise_magnitude: 12.,
    noise_scale: 4.,
    noise_layers: 3,
};

pub const DEFAULT_FRAG_SETTINGS: FragSettings = FragSettings {
    use_ray_tracing: false,
    _pad0: [0; 3],
};

pub const DEFAULT_POSTPROCESSING: Postprocessing = Postprocessing {
    exposure: 1.,
    temperature: 0.,
    tint: 0.,
    contrast: 1.,
    brightness: 0.,
    color_filter: Vector3::new(1., 1., 1.),
    saturation: 1.,
    tonemapper: Tonemapper::HillAces,
    gamma: 1.,
};
