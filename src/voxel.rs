mod binary_cube;
pub mod coordinates;
mod culled_meshing;
pub mod gpu_memory;
mod greedy_meshing;
pub mod meshlet;
mod sparse_octree;
mod square_invariant;
pub mod vertex;

use crate::config::{
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
};
use crate::interface::EnumInterface;
use crate::mesh::MeshData;
use crate::voxel::culled_meshing::CulledMeshing;
use crate::voxel::gpu_memory::VoxelGpuMemory;
use crate::voxel::greedy_meshing::GreedyMeshing;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::square_invariant::SquareInvariant;
use crate::voxel::vertex::VoxelVertex;
use bracket_noise::prelude::*;
use nalgebra::{DMatrix, Vector2, Vector3};
use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

trait MeshingAlgorithm {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> MeshData<VoxelVertex>;
}

trait ChunkPriorityAlgorithm {
    fn select(&mut self) -> Option<Vector3<i64>>;

    fn update_camera(&mut self, camera: Vector3<i64>);

    fn clear(
        &mut self,
        camera: Vector3<i64>,
        render_distance_horizontal: i64,
        render_distance_vertical: i64,
    );
}

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
    chunk_priority: SquareInvariant,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum MeshingAlgorithmKind {
    Culled,
    Greedy,
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
                chunk_priority: SquareInvariant::new(
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

impl VoxelKind {
    pub fn is_air(&self) -> bool {
        matches!(self, VoxelKind::Air)
    }
}

impl EnumInterface for MeshingAlgorithmKind {
    const VALUES: &'static [MeshingAlgorithmKind] =
        &[MeshingAlgorithmKind::Culled, MeshingAlgorithmKind::Greedy];

    fn label(&self) -> Cow<str> {
        match self {
            MeshingAlgorithmKind::Culled => Cow::Borrowed("Culled Meshing"),
            MeshingAlgorithmKind::Greedy => Cow::Borrowed("Greedy Meshing"),
        }
    }
}

fn voxel_thread(shared: &VoxelsShared) {
    let mut state = shared.state.lock().unwrap();
    loop {
        if state.shutdown {
            break;
        }

        let config = state.config.clone();
        let config_generation = state.config_generation;
        let noise = state.heightmap_noise.clone();

        state
            .chunk_priority
            .update_camera(*shared.camera.lock().unwrap());
        let Some(chunk) = state.chunk_priority.select() else {
            state = shared.wake.wait(state).unwrap();
            continue;
        };

        let mut offsets = vec![Vector3::zeros()];
        offsets.extend_from_slice(&DIRECTIONS);
        let mut all_svos = Vec::new();
        for offset in offsets {
            let chunk = chunk + offset;
            let svo = if let Some(svo) = state.loaded_svos.get(&chunk) {
                svo.clone()
            } else {
                let heightmap = if let Some(heightmap) = state.loaded_heightmaps.get(&chunk.xy()) {
                    heightmap.clone()
                } else {
                    drop(state);
                    let heightmap = Arc::new(generate_heightmap(chunk, &noise, &config));
                    state = shared.state.lock().unwrap();
                    state
                        .loaded_heightmaps
                        .insert(chunk.xy(), heightmap.clone());
                    heightmap
                };
                drop(state);
                let chunk_svo = Arc::new(generate_svo(chunk, &heightmap, &config));
                state = shared.state.lock().unwrap();
                state.loaded_svos.insert(chunk, chunk_svo.clone());
                chunk_svo
            };
            all_svos.push(svo);
        }
        drop(state);
        let chunk_svo = &all_svos[0];
        let neighbour_svos = std::array::from_fn(|i| &*all_svos[i + 1]);
        let raw_mesh = generate_mesh(chunk_svo, neighbour_svos, &config);
        let mesh = meshlet::from_unclustered_mesh(&raw_mesh);
        state = shared.state.lock().unwrap();
        if config_generation != state.config_generation {
            continue;
        }
        state.gpu_memory.upload_meshlet(chunk, mesh);
    }
}

fn generate_heightmap(
    chunk: Vector3<i64>,
    noise: &FastNoise,
    config: &VoxelsConfig,
) -> DMatrix<i64> {
    let chunk_coordinates = chunk.xy() * config.chunk_size as i64;
    let mut heightmap = DMatrix::from_element(config.chunk_size, config.chunk_size, 0);
    for x in 0..config.chunk_size {
        for y in 0..config.chunk_size {
            let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
            let noise_position = column_coordinates.cast::<f32>() * config.heightmap_frequency;
            let raw_noise = noise.get_noise(noise_position.x, noise_position.y);
            let scaled_noise = (raw_noise + config.heightmap_bias) * config.heightmap_amplitude;
            heightmap[(x, y)] = scaled_noise.round() as i64;
        }
    }
    heightmap
}

fn generate_svo(
    chunk: Vector3<i64>,
    heightmap: &DMatrix<i64>,
    config: &VoxelsConfig,
) -> SparseOctree {
    assert_eq!(heightmap.nrows(), config.chunk_size);
    assert_eq!(heightmap.ncols(), config.chunk_size);
    svo_from_heightmap_impl(
        0,
        0,
        chunk.z * config.chunk_size as i64,
        config.chunk_size,
        heightmap,
    )
}

fn svo_from_heightmap_impl(
    x: usize,
    y: usize,
    z: i64,
    n: usize,
    heightmap: &DMatrix<i64>,
) -> SparseOctree {
    'check_all_same: {
        let material = material_from_height(heightmap[(x, y)], z);
        for ly in y..y + n {
            for lx in x..x + n {
                let height = heightmap[(lx, ly)];
                let low_material = material_from_height(height, z);
                let high_material = material_from_height(height, z + n as i64 - 1);
                if low_material != material || high_material != material {
                    break 'check_all_same;
                }
            }
        }
        return SparseOctree::Uniform { kind: material };
    }
    let mut children = MaybeUninit::uninit_array();
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                children[4 * dz + 2 * dy + dx].write(svo_from_heightmap_impl(
                    x + dx * n / 2,
                    y + dy * n / 2,
                    z + dz as i64 * n as i64 / 2,
                    n / 2,
                    heightmap,
                ));
            }
        }
    }
    let children = Box::new(unsafe { MaybeUninit::array_assume_init(children) });
    SparseOctree::Mixed { children }
}

fn material_from_height(height: i64, z: i64) -> VoxelKind {
    if height <= z {
        VoxelKind::Air
    } else if height <= z + 1 {
        VoxelKind::Grass
    } else if height <= z + 5 {
        VoxelKind::Dirt
    } else {
        VoxelKind::Stone
    }
}

fn generate_mesh(
    chunk_svo: &SparseOctree,
    neighbour_svos: [&SparseOctree; 6],
    config: &VoxelsConfig,
) -> MeshData<VoxelVertex> {
    if chunk_svo.is_uniform()
        && neighbour_svos
            .iter()
            .all(|neighbour_svo| *neighbour_svo == chunk_svo)
    {
        return MeshData {
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    let meshing_algorithm = match config.meshing_algorithm {
        MeshingAlgorithmKind::Culled => CulledMeshing::mesh,
        MeshingAlgorithmKind::Greedy => GreedyMeshing::mesh,
    };
    let mesh = meshing_algorithm(chunk_svo, neighbour_svos, config.chunk_size);
    mesh.remove_duplicate_vertices()
}

fn chunk_from_position(position: Vector3<f32>, chunk_size: usize) -> Vector3<i64> {
    position.map(|coord| coord.div_euclid(chunk_size as f32) as i64)
}
