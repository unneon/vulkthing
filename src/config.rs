use crate::camera::first_person::FirstPersonCamera;
use crate::renderer::uniform::Tonemapper;
use crate::renderer::{PostprocessSettings, RendererSettings};
use crate::voxels::MeshingAlgorithmKind;
use nalgebra::Vector3;

pub const DEFAULT_SUN_POSITION: Vector3<f32> = Vector3::new(0., 0., DEFAULT_SUN_RADIUS);

pub const DEFAULT_SUN_RADIUS: f32 = 2000.;

pub const DEFAULT_SUN_SPEED: f32 = 0.1;

pub const DEFAULT_STAR_COUNT: usize = 2048;
pub const DEFAULT_STAR_RADIUS: f32 = 30000.;
pub const DEFAULT_STAR_MIN_SCALE: f32 = 10.;
pub const DEFAULT_STAR_MAX_SCALE: f32 = 100.;

pub const DEFAULT_CAMERA: FirstPersonCamera = FirstPersonCamera {
    position: Vector3::new(0., 0., 20.),
    walk_direction: Vector3::new(0., 0., 0.),
    pitch: 0.,
    yaw: 0.,
};

pub const DEFAULT_RENDERER_SETTINGS: RendererSettings = RendererSettings {
    atmosphere_in_scattering_samples: 10,
    atmosphere_optical_depth_samples: 3,
    atmosphere_wavelengths: Vector3::new(700., 530., 440.),
    depth_near: 0.2,
    depth_far: 65536.,
    enable_atmosphere: true,
    postprocess: PostprocessSettings {
        exposure: 1.,
        bloom_exponent_coefficient: 0.25,
        bloom_radius: 3,
        bloom_strength: 0.01,
        bloom_threshold: 1.,
        temperature: 0.,
        tint: 0.,
        contrast: 1.,
        brightness: 0.,
        color_filter: Vector3::new(1., 1., 1.),
        saturation: 1.,
        tonemapper: Tonemapper::HillAces,
        gamma: 1.,
    },
};

pub const DEFAULT_VOXEL_CHUNK_SIZE: usize = 32;
pub const DEFAULT_VOXEL_HEIGHTMAP_AMPLITUDE: f32 = 32.;
pub const DEFAULT_VOXEL_HEIGHTMAP_FREQUENCY: f32 = 0.01;
pub const DEFAULT_VOXEL_HEIGHTMAP_BIAS: f32 = 0.;
pub const DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL: usize = 1024;
pub const DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL: usize = 1024;
pub const DEFAULT_VOXEL_MESHING_ALGORITHM: MeshingAlgorithmKind = MeshingAlgorithmKind::Greedy;
pub const DEFAULT_VOXEL_VERTEX_MEMORY: usize = 2048 * 1024 * 1024;
