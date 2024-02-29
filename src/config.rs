use crate::camera::first_person::FirstPersonCamera;
use crate::renderer::uniform::Tonemapper;
use crate::renderer::{PostprocessSettings, RendererSettings};
use crate::voxel::meshing::MeshingAlgorithmKind;
use crate::voxel::VoxelsConfig;
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
        tonemapper: Tonemapper::HillAces,
        gamma: 1.,
    },
};

pub const DEFAULT_VOXEL_CONFIG: VoxelsConfig = VoxelsConfig {
    seed: 907,
    chunk_size: 64,
    heightmap_amplitude: 32.,
    heightmap_frequency: 0.01,
    heightmap_bias: 0.,
    render_distance_horizontal: 1024,
    render_distance_vertical: 64,
    meshing_algorithm: MeshingAlgorithmKind::Greedy,
};
pub const DEFAULT_VOXEL_INDEX_MAX_COUNT: usize = 3 * 256 * DEFAULT_VOXEL_MESHLET_MAX_COUNT;
pub const DEFAULT_VOXEL_MESHLET_MAX_COUNT: usize = 1024 * 1024;
pub const DEFAULT_VOXEL_VERTEX_MAX_COUNT: usize = 128 * DEFAULT_VOXEL_MESHLET_MAX_COUNT;
