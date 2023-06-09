use crate::camera::first_person::FirstPersonCamera;
use crate::grass::Grass;
use crate::planet::{NoiseType, Planet};
use crate::renderer::uniform::{FragSettings, Postprocessing, Tonemapper};
use crate::renderer::RendererSettings;
use nalgebra::Vector3;
use std::f32::consts::FRAC_PI_2;

pub const DEFAULT_PLANET: Planet = Planet {
    resolution: 400,
    radius: 1000.,
    noise_type: NoiseType::Ridge,
    noise_magnitude: 50.,
    noise_scale: 1.,
    noise_layers: 3,
};

pub const DEFAULT_SUN_POSITION: Vector3<f32> = Vector3::new(0., 0., DEFAULT_PLANET.radius + 100.);

pub const DEFAULT_GRASS: Grass = Grass {
    blades_per_triangle: 128,
    height_average: 1.2,
    height_max_variance: 0.3,
    height_noise_frequency: 0.15,
    width: 0.15,
    sway_frequency: 1.,
    sway_amplitude: 0.2,
    chunk_count: 1023,
    chunk_load_distance: 400.,
    chunk_unload_distance: 450.,
};

pub const DEFAULT_CAMERA: FirstPersonCamera = FirstPersonCamera {
    position: Vector3::new(0., 0., 5000.),
    walk_direction: Vector3::new(0., 0., 0.),
    pitch: -FRAC_PI_2,
    yaw: 0.,
};

pub const DEFAULT_RENDERER_SETTINGS: RendererSettings = RendererSettings {
    depth_near: 0.05,
    depth_far: 200000.,
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
