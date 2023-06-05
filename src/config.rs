use crate::camera::Camera;
use crate::grass::Grass;
use crate::planet::{NoiseType, Planet};
use crate::renderer::uniform::{FragSettings, Postprocessing, Tonemapper};
use nalgebra::{Quaternion, Unit, Vector3};

pub const DEFAULT_PLANET: Planet = Planet {
    resolution: 400,
    radius: 1000.,
    noise_type: NoiseType::Ridge,
    noise_magnitude: 50.,
    noise_scale: 1.,
    noise_layers: 3,
};

pub const DEFAULT_SUN_POSITION: Vector3<f32> = Vector3::new(
    -2. * DEFAULT_PLANET.radius,
    2. * DEFAULT_PLANET.radius,
    2. * DEFAULT_PLANET.radius,
);

pub const DEFAULT_GRASS: Grass = Grass {
    render_distance: 100.,
    blades_per_triangle: 1024,
    height_average: 0.7,
    height_max_variance: 0.5,
    height_noise_frequency: 0.15,
    width: 0.1,
};

pub const DEFAULT_CAMERA: Camera = Camera {
    position: Vector3::new(
        0.,
        0.,
        DEFAULT_PLANET.radius + DEFAULT_PLANET.noise_magnitude + 1.5,
    ),
    rotation: Unit::new_unchecked(Quaternion::new(1., 0., 0., 0.)),
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
