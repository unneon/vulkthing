mod binary_cube;
mod culled_meshing;
mod greedy_meshing;
mod meshlet;
mod sparse_octree;

use crate::config::{
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_HEIGHTMAP_AMPLITUDE, DEFAULT_VOXEL_HEIGHTMAP_BIAS,
    DEFAULT_VOXEL_HEIGHTMAP_FREQUENCY, DEFAULT_VOXEL_HEIGHTMAP_NOISE_IMPLEMENTATION,
    DEFAULT_VOXEL_MESHING_ALGORITHM, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
};
use crate::interface::EnumInterface;
use crate::mesh::MeshData;
use crate::renderer::uniform::VoxelMeshlet;
use crate::renderer::vertex::VoxelVertex;
use crate::voxels::culled_meshing::CulledMeshing;
use crate::voxels::greedy_meshing::GreedyMeshing;
use crate::voxels::sparse_octree::SparseOctree;
use bracket_noise::prelude::*;
use nalgebra::{DMatrix, Vector2, Vector3};
use noise::Perlin;
use noise::{NoiseFn, Seedable};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};

trait MeshingAlgorithm {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> MeshData<VoxelVertex>;
}

pub struct Voxels<'a> {
    config: VoxelsConfig,
    config_rx: mpsc::Receiver<VoxelsConfig>,
    config_changed: bool,
    shutdown: Arc<AtomicBool>,
    heightmap_noise: Perlin,
    heightmap_noise_bracket: FastNoise,
    // TODO: That is really not in the spirit of Rust safety.
    vertex_buffer: &'a mut [MaybeUninit<VoxelVertex>],
    index_buffer: &'a mut [MaybeUninit<u32>],
    meshlet_buffer: &'a mut [MaybeUninit<VoxelMeshlet>],
    vertices: Arc<AtomicU64>,
    triangles: Arc<AtomicU64>,
    meshlets: Arc<AtomicU64>,
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

impl<'a> Voxels<'a> {
    pub fn new(
        seed: u64,
        vertex_buffer: &'a mut [MaybeUninit<VoxelVertex>],
        index_buffer: &'a mut [MaybeUninit<u32>],
        meshlet_buffer: &'a mut [MaybeUninit<VoxelMeshlet>],
    ) -> (Voxels<'a>, mpsc::Sender<VoxelsConfig>) {
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
            heightmap_noise: Perlin::new(seed as u32),
            heightmap_noise_bracket: FastNoise::seeded(seed),
            vertex_buffer,
            index_buffer,
            meshlet_buffer,
            vertices: Arc::new(AtomicU64::new(0)),
            triangles: Arc::new(AtomicU64::new(0)),
            meshlets: Arc::new(AtomicU64::new(0)),
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
            self.heightmap_noise = Perlin::new(self.heightmap_noise.seed());
            self.vertices.store(0, Ordering::SeqCst);
            self.loaded_cpu.clear();
            self.loaded_gpu.clear();
            self.loaded_heightmaps.clear();
        }
        let camera_chunk = Vector3::new(
            (camera.x / self.config.chunk_size as f32).floor() as i64,
            (camera.y / self.config.chunk_size as f32).floor() as i64,
            (camera.z / self.config.chunk_size as f32).floor() as i64,
        );
        let distance_horizontal =
            (self.config.render_distance_horizontal / self.config.chunk_size) as i64 + 1;
        let distance_vertical =
            (self.config.render_distance_vertical / self.config.chunk_size) as i64 + 1;
        let range_horizontal = -distance_horizontal..=distance_horizontal;
        let range_vertical = -distance_vertical..=distance_vertical;
        let mut to_load = Vec::new();
        for dx in range_horizontal.clone() {
            for dy in range_horizontal.clone() {
                for dz in range_vertical.clone() {
                    let chunk = camera_chunk + Vector3::new(dx, dy, dz);
                    if !self.loaded_gpu.contains(&chunk) {
                        to_load.push(chunk);
                    }
                }
            }
        }
        to_load.sort_by_key(|chunk| (chunk - camera_chunk).abs().sum());
        for chunk in to_load {
            self.load_svo_cpu(chunk);
            for dir in DIRECTIONS {
                let neighbour = chunk + dir;
                self.load_svo_cpu(neighbour);
            }
            self.load_mesh_gpu(chunk);
            if self.check_config_change() || self.shutdown.load(Ordering::SeqCst) {
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
        let mut mesh = meshlet::from_unclustered_mesh(&mesh);

        let old_vertex_count = self.vertices.load(Ordering::SeqCst) as usize;
        let old_triangle_count = self.triangles.load(Ordering::SeqCst) as usize;
        let old_meshlet_count = self.meshlets.load(Ordering::SeqCst) as usize;

        for meshlet in &mut mesh.meshlets {
            meshlet.vertex_offset += old_vertex_count as u32;
            meshlet.triangle_offset += old_triangle_count as u32;
        }
        for vertex in &mut mesh.vertices {
            vertex.position += (chunk * self.config.chunk_size as i64).cast::<f32>();
        }
        // Triangles are local to the meshlet, so it's unnecessary to fix them.

        let new_vertex_count = old_vertex_count + mesh.vertices.len();
        let new_triangle_count = old_triangle_count + mesh.triangles.len();
        let new_meshlet_count = old_meshlet_count + mesh.meshlets.len();

        let vertex_memory = &mut self.vertex_buffer[old_vertex_count..new_vertex_count];
        let index_memory = &mut self.index_buffer[old_triangle_count..new_triangle_count];
        let meshlet_memory = &mut self.meshlet_buffer[old_meshlet_count..new_meshlet_count];

        MaybeUninit::write_slice(vertex_memory, &mesh.vertices);
        MaybeUninit::write_slice(index_memory, &mesh.triangles);
        MaybeUninit::write_slice(meshlet_memory, &mesh.meshlets);

        self.vertices
            .store(new_vertex_count as u64, Ordering::SeqCst);
        self.triangles
            .store(new_triangle_count as u64, Ordering::SeqCst);
        self.meshlets
            .store(new_meshlet_count as u64, Ordering::SeqCst);
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

    pub fn shared_vertex_count(&self) -> Arc<AtomicU64> {
        self.vertices.clone()
    }

    pub fn shared_index_count(&self) -> Arc<AtomicU64> {
        self.triangles.clone()
    }

    pub fn shared_meshlet_count(&self) -> Arc<AtomicU64> {
        self.meshlets.clone()
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
