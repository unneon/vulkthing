mod binary_cube;
mod chunk_priority;
pub mod gpu_memory;
mod local_mesh;
pub mod material;
pub mod meshing;
pub mod meshlet;
mod sparse_octree;
mod thread;
mod world_generation;

use crate::config::{
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
};
use crate::voxel::chunk_priority::{ChunkPriority, ChunkPriorityAlgorithm};
use crate::voxel::gpu_memory::VoxelGpuMemory;
use crate::voxel::meshing::MeshingAlgorithmKind;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::thread::voxel_thread;
use bracket_noise::prelude::{FastNoise, NoiseType};
use nalgebra::{DMatrix, Vector2, Vector3};
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub struct Voxels {
    shared: Arc<VoxelsShared>,
    handles: Vec<JoinHandle<()>>,
    // TODO: Deduplicate?
    camera: Vector3<i64>,
    config: VoxelsConfig,
}

pub struct VoxelsShared {
    camera: Mutex<Vector3<i64>>,
    state: Mutex<VoxelsState>,
    wake: Condvar,
}

pub struct VoxelsState {
    chunk_priority: ChunkPriority,
    heightmap_noise: Arc<FastNoise>,
    loaded_svos: HashMap<Vector3<i64>, Arc<SparseOctree>>,
    loaded_heightmaps: HashMap<Vector2<i64>, Arc<DMatrix<i64>>>,
    gpu_memory: VoxelGpuMemory,
    config: VoxelsConfig,
    config_generation: u64,
    shutdown: bool,
}

#[derive(Clone)]
pub struct VoxelsConfig {
    pub seed: u64,
    pub chunk_size: usize,
    pub heightmap_amplitude: f32,
    pub heightmap_frequency: f32,
    pub heightmap_bias: f32,
    pub render_distance_horizontal: usize,
    pub render_distance_vertical: usize,
    pub meshing_algorithm: MeshingAlgorithmKind,
}

pub const DIRECTIONS: [Vector3<i64>; 6] = [
    Vector3::new(1, 0, 0),
    Vector3::new(-1, 0, 0),
    Vector3::new(0, 1, 0),
    Vector3::new(0, -1, 0),
    Vector3::new(0, 0, 1),
    Vector3::new(0, 0, -1),
];

impl Voxels {
    pub fn new(
        config: VoxelsConfig,
        camera: Vector3<f32>,
        gpu_memory: VoxelGpuMemory,
        thread_count: usize,
    ) -> Voxels {
        let camera = chunk_from_position(camera, DEFAULT_VOXEL_CHUNK_SIZE);
        let mut noise = FastNoise::seeded(config.seed);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(1.);
        let shared = Arc::new(VoxelsShared {
            camera: Mutex::new(camera),
            state: Mutex::new(VoxelsState {
                chunk_priority: ChunkPriority::new(
                    camera,
                    DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL.div_ceil(DEFAULT_VOXEL_CHUNK_SIZE)
                        as i64,
                    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL.div_ceil(DEFAULT_VOXEL_CHUNK_SIZE)
                        as i64,
                ),
                heightmap_noise: Arc::new(noise),
                loaded_svos: HashMap::new(),
                loaded_heightmaps: HashMap::new(),
                gpu_memory,
                config: config.clone(),
                config_generation: 0,
                shutdown: false,
            }),
            wake: Condvar::new(),
        });
        let mut handles = Vec::new();
        for _ in 0..thread_count {
            let shared = shared.clone();
            handles.push(std::thread::spawn(move || voxel_thread(&shared)));
        }
        Voxels {
            shared,
            handles,
            camera,
            config,
        }
    }

    pub fn update_camera(&self, new_position: Vector3<f32>) {
        let new_chunk = chunk_from_position(new_position, self.config.chunk_size);
        let mut camera = self.shared.camera.lock().unwrap();
        let old_chunk = *camera;
        *camera = new_chunk;
        drop(camera);
        if new_chunk != old_chunk {
            self.shared.wake.notify_all();
        }
    }

    pub fn update_config(&self, new_config: VoxelsConfig) {
        let mut state = self.shared.state.lock().unwrap();
        state.chunk_priority.clear(
            self.camera,
            new_config
                .render_distance_horizontal
                .div_ceil(new_config.chunk_size) as i64,
            new_config
                .render_distance_vertical
                .div_ceil(new_config.chunk_size) as i64,
        );
        let mut noise = FastNoise::seeded(new_config.seed);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_frequency(1.);
        state.heightmap_noise = Arc::new(noise);
        state.loaded_svos.clear();
        state.loaded_heightmaps.clear();
        state.gpu_memory.clear();
        state.config = new_config;
        state.config_generation += 1;
        drop(state);
        self.shared.wake.notify_all();
    }

    pub fn shutdown(self) {
        self.shared.state.lock().unwrap().shutdown = true;
        self.shared.wake.notify_all();
        for handle in self.handles {
            handle.join().unwrap();
        }
        self.shared.state.lock().unwrap().gpu_memory.cleanup();
    }
}

fn chunk_from_position(position: Vector3<f32>, chunk_size: usize) -> Vector3<i64> {
    position.map(|coord| coord.div_euclid(chunk_size as f32) as i64)
}
