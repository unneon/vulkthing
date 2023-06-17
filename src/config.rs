use crate::camera::first_person::FirstPersonCamera;
use crate::grass::Grass;
use crate::planet::{NoiseType, Planet};
use crate::renderer::uniform::{Atmosphere, FragSettings, Gaussian, Postprocessing, Tonemapper};
use crate::renderer::RendererSettings;
use ash::vk;
use nalgebra::Vector3;

pub const DEFAULT_PLANET: Planet = Planet {
    resolution: 400,
    noise_type: NoiseType::Ridge,
    noise_magnitude: 0.05,
    noise_scale: 1.,
    noise_layers: 3,
};

pub const DEFAULT_PLANET_POSITION: Vector3<f32> = Vector3::new(0., 0., 0.);

pub const DEFAULT_PLANET_SCALE: f32 = 1000.;

pub const DEFAULT_SUN_POSITION: Vector3<f32> = Vector3::new(0., 0., DEFAULT_SUN_RADIUS);

pub const DEFAULT_SUN_RADIUS: f32 = 2. * DEFAULT_PLANET_SCALE;

pub const DEFAULT_SUN_SPEED: f32 = 0.1;

pub const DEFAULT_STAR_COUNT: usize = 2048;
pub const DEFAULT_STAR_RADIUS: f32 = 30. * DEFAULT_PLANET_SCALE;
pub const DEFAULT_STAR_MIN_SCALE: f32 = 10.;
pub const DEFAULT_STAR_MAX_SCALE: f32 = 100.;
pub const DEFAULT_STAR_MIN_EMIT: f32 = 0.1;
pub const DEFAULT_STAR_MAX_EMIT: f32 = 10.;

pub const DEFAULT_GRASS: Grass = Grass {
    blades_per_triangle: 128,
    height_average: 1.2,
    height_max_variance: 0.3,
    height_noise_frequency: 0.15,
    width: 0.15,
    sway_frequency: 1.,
    sway_amplitude: 0.2,
    chunk_count: 1023,
    chunk_load_distance: 0.4 * DEFAULT_PLANET_SCALE,
    chunk_unload_distance: 0.45 * DEFAULT_PLANET_SCALE,
};

pub const DEFAULT_CAMERA: FirstPersonCamera = FirstPersonCamera {
    position: Vector3::new(
        0.,
        0.,
        DEFAULT_PLANET_SCALE * (1. + 1.5 * DEFAULT_PLANET.noise_magnitude),
    ),
    walk_direction: Vector3::new(0., 0., 0.),
    pitch: 0.,
    yaw: 0.,
};

pub const DEFAULT_RENDERER_SETTINGS: RendererSettings = RendererSettings {
    depth_near: 0.2,
    depth_far: 65536.,
    msaa_samples: vk::SampleCountFlags::TYPE_2,
};

pub const DEFAULT_FRAG_SETTINGS: FragSettings = FragSettings {
    use_ray_tracing: true,
    _pad0: [0; 3],
};

pub const DEFAULT_ATMOSPHERE: Atmosphere = Atmosphere {
    enable: true,
    _pad0: [0; 3],
    scatter_point_count: 10,
    optical_depth_point_count: 3,
    density_falloff: 6.,
    scale: 1.3,
    planet_radius: DEFAULT_PLANET_SCALE,
    planet_position: DEFAULT_PLANET_POSITION,
    sun_position: DEFAULT_SUN_POSITION,
    wavelengths: Vector3::new(700., 530., 440.),
    scattering_strength: 0.01,
    henyey_greenstein_g: 0.,
};

pub const DEFAULT_GAUSSIAN: Gaussian = Gaussian {
    threshold: 1.,
    radius: 2,
    exponent_coefficient: 0.7,
};

pub const DEFAULT_POSTPROCESSING: Postprocessing = Postprocessing {
    exposure: 1.,
    bloom: 1.,
    temperature: 0.,
    tint: 0.,
    contrast: 1.,
    brightness: 0.,
    color_filter: Vector3::new(1., 1., 1.),
    saturation: 1.,
    tonemapper: Tonemapper::HillAces,
    gamma: 1.,
};
