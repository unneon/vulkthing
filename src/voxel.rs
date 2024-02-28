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
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_HEIGHTMAP_AMPLITUDE, DEFAULT_VOXEL_HEIGHTMAP_BIAS,
    DEFAULT_VOXEL_HEIGHTMAP_FREQUENCY, DEFAULT_VOXEL_HEIGHTMAP_NOISE_IMPLEMENTATION,
    DEFAULT_VOXEL_MESHING_ALGORITHM, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
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
use noise::Perlin;
use noise::{NoiseFn, Seedable};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

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
        render_distance_horizontal: usize,
        render_distance_vertical: usize,
    );
}

/*

   voxel main loop state management from lib.rs
   - definitely prepare an easier interface
   voxel main loop state management from thread(s)
   - required support for multiple threads
   - cancel further generation early on camera position changes?
   voxel chunk scanning algorithm
   - smarter algorithm?
   voxel meshing
   - already extracted, almost
   voxel gpu memory management
   - definitely extract into separate file
   - global device variable?
   voxel renderer synchronization
   - through Arcs or through main loop?
   voxel caching

*/

pub struct Voxels {
    config: VoxelsConfig,
    config_rx: mpsc::Receiver<VoxelsConfig>,
    config_changed: bool,
    shutdown: Arc<AtomicBool>,
    camera_update: Arc<AtomicBool>,
    chunk_priority: SquareInvariant,
    heightmap_noise: Perlin,
    heightmap_noise_bracket: FastNoise,
    gpu_memory: Arc<VoxelGpuMemory>,
    pub loaded_cpu: HashMap<Vector3<i64>, SparseOctree>,
    loaded_gpu: HashSet<Vector3<i64>>,
    loaded_heightmaps: HashMap<Vector2<i64>, DMatrix<i64>>,
}

#[derive(Clone)]
pub struct VoxelsConfig {
    pub chunk_size: usize,
    pub heightmap_amplitude: f32,
    pub heightmap_frequency: f32,
    pub heightmap_bias: f32,
    pub heightmap_noise_implementation: NoiseImplementation,
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
pub enum NoiseImplementation {
    BracketNoise,
    Noise,
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
        seed: u64,
        camera: Vector3<f32>,
        gpu_memory: Arc<VoxelGpuMemory>,
    ) -> (Voxels, mpsc::Sender<VoxelsConfig>) {
        let (new_config_tx, new_config_rx) = mpsc::channel();
        let mut voxels = Voxels {
            config: VoxelsConfig {
                chunk_size: DEFAULT_VOXEL_CHUNK_SIZE,
                heightmap_amplitude: DEFAULT_VOXEL_HEIGHTMAP_AMPLITUDE,
                heightmap_frequency: DEFAULT_VOXEL_HEIGHTMAP_FREQUENCY,
                heightmap_bias: DEFAULT_VOXEL_HEIGHTMAP_BIAS,
                heightmap_noise_implementation: DEFAULT_VOXEL_HEIGHTMAP_NOISE_IMPLEMENTATION,
                render_distance_horizontal: DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
                render_distance_vertical: DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
                meshing_algorithm: DEFAULT_VOXEL_MESHING_ALGORITHM,
            },
            config_rx: new_config_rx,
            config_changed: true,
            shutdown: Arc::new(AtomicBool::new(false)),
            camera_update: Arc::new(AtomicBool::new(false)),
            chunk_priority: SquareInvariant::new(
                chunk_from_position(camera, DEFAULT_VOXEL_CHUNK_SIZE),
                DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL.div_ceil(DEFAULT_VOXEL_CHUNK_SIZE),
                DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL.div_ceil(DEFAULT_VOXEL_CHUNK_SIZE),
            ),
            heightmap_noise: Perlin::new(seed as u32),
            heightmap_noise_bracket: FastNoise::seeded(seed),
            gpu_memory,
            loaded_cpu: HashMap::new(),
            loaded_gpu: HashSet::new(),
            loaded_heightmaps: HashMap::new(),
        };
        voxels
            .heightmap_noise_bracket
            .set_noise_type(NoiseType::Perlin);
        voxels.heightmap_noise_bracket.set_frequency(1.);
        (voxels, new_config_tx)
    }

    pub fn update_camera(&mut self, camera: Vector3<f32>) {
        if self.config_changed {
            self.config_changed = false;
            self.chunk_priority.clear(
                chunk_from_position(camera, self.config.chunk_size),
                self.config
                    .render_distance_horizontal
                    .div_ceil(self.config.chunk_size),
                self.config
                    .render_distance_vertical
                    .div_ceil(self.config.chunk_size),
            );
            self.heightmap_noise = Perlin::new(self.heightmap_noise.seed());
            self.gpu_memory.clear();
            self.loaded_cpu.clear();
            self.loaded_gpu.clear();
            self.loaded_heightmaps.clear();
        }
        self.chunk_priority
            .update_camera(chunk_from_position(camera, self.config.chunk_size));
        while let Some(chunk) = self.chunk_priority.select() {
            self.load_svo_cpu(chunk);
            for dir in DIRECTIONS {
                let neighbour = chunk + dir;
                self.load_svo_cpu(neighbour);
            }
            self.load_mesh_gpu(chunk);
            if self.check_config_change()
                || self.shutdown.load(Ordering::SeqCst)
                || self.camera_update.load(Ordering::SeqCst)
            {
                break;
            }
        }
    }

    pub fn load_svo_cpu(&mut self, chunk: Vector3<i64>) {
        if self.loaded_cpu.contains_key(&chunk) {
            return;
        }
        self.load_heightmap_cpu(chunk);
        let heightmap = &self.loaded_heightmaps[&chunk.xy()];
        let svo = self.generate_chunk_svo(chunk, heightmap);
        self.loaded_cpu.insert(chunk, svo);
    }

    pub fn load_mesh_gpu(&mut self, chunk: Vector3<i64>) {
        assert!(!self.loaded_gpu.contains(&chunk));
        let chunk_svo = &self.loaded_cpu[&chunk];
        let neighbour_svos = std::array::from_fn(|i| &self.loaded_cpu[&(chunk + DIRECTIONS[i])]);
        let mesh = self.generate_chunk_mesh(chunk_svo, neighbour_svos);
        let mesh = meshlet::from_unclustered_mesh(&mesh);
        self.gpu_memory.upload_meshlet(chunk, mesh);
        self.loaded_gpu.insert(chunk);
    }

    pub fn load_heightmap_cpu(&mut self, chunk: Vector3<i64>) {
        if self.loaded_heightmaps.contains_key(&chunk.xy()) {
            return;
        }
        let heightmap = self.generate_heightmap_bracket_noise(chunk);
        self.loaded_heightmaps.insert(chunk.xy(), heightmap);
    }

    pub fn generate_heightmap_noise(&mut self, chunk: Vector3<i64>) -> DMatrix<i64> {
        let chunk_coordinates = chunk.xy() * self.config.chunk_size as i64;
        let mut heightmap =
            DMatrix::from_element(self.config.chunk_size, self.config.chunk_size, 0);
        for x in 0..self.config.chunk_size {
            for y in 0..self.config.chunk_size {
                let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
                let noise_position =
                    column_coordinates.cast::<f64>() * self.config.heightmap_frequency as f64;
                let noise_arguments: [f64; 2] = noise_position.into();
                let raw_noise = self.heightmap_noise.get(noise_arguments) as f32;
                let scaled_noise =
                    (raw_noise + self.config.heightmap_bias) * self.config.heightmap_amplitude;
                heightmap[(x, y)] = scaled_noise.round() as i64;
            }
        }
        heightmap
    }

    pub fn generate_heightmap_bracket_noise(&mut self, chunk: Vector3<i64>) -> DMatrix<i64> {
        let chunk_coordinates = chunk.xy() * self.config.chunk_size as i64;
        let mut heightmap =
            DMatrix::from_element(self.config.chunk_size, self.config.chunk_size, 0);
        for x in 0..self.config.chunk_size {
            for y in 0..self.config.chunk_size {
                let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
                let noise_position =
                    column_coordinates.cast::<f32>() * self.config.heightmap_frequency;
                let raw_noise = self
                    .heightmap_noise_bracket
                    .get_noise(noise_position.x, noise_position.y);
                let scaled_noise =
                    (raw_noise + self.config.heightmap_bias) * self.config.heightmap_amplitude;
                heightmap[(x, y)] = scaled_noise.round() as i64;
            }
        }
        heightmap
    }

    pub fn generate_chunk_svo(
        &self,
        chunk: Vector3<i64>,
        heightmap: &DMatrix<i64>,
    ) -> SparseOctree {
        assert_eq!(heightmap.nrows(), self.config.chunk_size);
        assert_eq!(heightmap.ncols(), self.config.chunk_size);
        svo_from_heightmap_impl(
            0,
            0,
            chunk.z * self.config.chunk_size as i64,
            self.config.chunk_size,
            heightmap,
        )
    }

    pub fn generate_chunk_mesh(
        &self,
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
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
        let meshing_algorithm = match self.config.meshing_algorithm {
            MeshingAlgorithmKind::Culled => CulledMeshing::mesh,
            MeshingAlgorithmKind::Greedy => GreedyMeshing::mesh,
        };
        let mesh = meshing_algorithm(chunk_svo, neighbour_svos, self.config.chunk_size);
        mesh.remove_duplicate_vertices()
    }

    fn check_config_change(&mut self) -> bool {
        let Ok(new_config) = self.config_rx.try_recv() else {
            return false;
        };
        self.config = new_config;
        self.config_changed = true;
        true
    }

    pub fn config(&self) -> &VoxelsConfig {
        &self.config
    }

    pub fn config_changed(&self) -> bool {
        self.config_changed
    }

    pub fn shutdown(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }

    pub fn camera_update(&self) -> Arc<AtomicBool> {
        self.camera_update.clone()
    }
}

impl VoxelKind {
    pub fn is_air(&self) -> bool {
        matches!(self, VoxelKind::Air)
    }
}

impl EnumInterface for NoiseImplementation {
    const VALUES: &'static [Self] = &[
        NoiseImplementation::Noise,
        NoiseImplementation::BracketNoise,
    ];

    fn label(&self) -> Cow<str> {
        Cow::Borrowed(match self {
            NoiseImplementation::BracketNoise => "bracket-noise",
            NoiseImplementation::Noise => "noise",
        })
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

fn chunk_from_position(position: Vector3<f32>, chunk_size: usize) -> Vector3<i64> {
    position.map(|coord| coord.div_euclid(chunk_size as f32) as i64)
}
