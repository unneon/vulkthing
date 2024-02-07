mod binary_cube;
mod sparse_octree;

use crate::config::{
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
};
use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use crate::voxels::binary_cube::BinaryCube;
use crate::voxels::sparse_octree::SparseOctree;
use log::debug;
use nalgebra::{DMatrix, Vector2, Vector3};
use noise::NoiseFn;
use noise::Perlin;
use rand::random;
use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Voxels<'a> {
    chunk_size: usize,
    heightmap_noise: Perlin,
    // TODO: That is really not in the spirit of Rust safety.
    buffer: &'a mut [MaybeUninit<Vertex>],
    vertices: Arc<AtomicU64>,
    loaded: HashMap<Vector3<i64>, SparseOctree>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Stone = 1,
}

const DIRECTIONS: [Vector3<i64>; 6] = [
    Vector3::new(1, 0, 0),
    Vector3::new(-1, 0, 0),
    Vector3::new(0, 1, 0),
    Vector3::new(0, -1, 0),
    Vector3::new(0, 0, 1),
    Vector3::new(0, 0, -1),
];

impl<'a> Voxels<'a> {
    pub fn new(buffer: &'a mut [MaybeUninit<Vertex>]) -> Voxels {
        Voxels {
            chunk_size: DEFAULT_VOXEL_CHUNK_SIZE,
            heightmap_noise: Perlin::new(random()),
            buffer,
            vertices: Arc::new(AtomicU64::new(0)),
            loaded: HashMap::new(),
        }
    }

    pub fn update_camera(&mut self, camera: Vector3<f32>) {
        let camera_chunk = Vector3::new(
            (camera.x / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
            (camera.y / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
            (camera.z / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
        );
        let distance_horizontal = DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL as i64;
        let distance_vertical = DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL as i64;
        let range_horizontal = -distance_horizontal..=distance_horizontal;
        let range_vertical = -distance_vertical..=distance_vertical;
        let mut to_load = Vec::new();
        for dx in range_horizontal.clone() {
            for dy in range_horizontal.clone() {
                for dz in range_vertical.clone() {
                    let chunk = camera_chunk + Vector3::new(dx, dy, dz);
                    if !self.loaded.contains_key(&chunk) {
                        to_load.push(chunk);
                    }
                }
            }
        }
        to_load.sort_by_key(|chunk| (chunk - camera_chunk).abs().sum());
        for chunk in to_load {
            self.generate_chunk_cpu(chunk);
            for dir in DIRECTIONS {
                let neighbour = chunk + dir;
                self.generate_chunk_cpu(neighbour);
            }
            self.generate_chunk_gpu(chunk);
        }
    }

    pub fn generate_chunk_cpu(&mut self, chunk: Vector3<i64>) {
        if self.loaded.contains_key(&chunk) {
            return;
        }
        let heightmap = self.generate_chunk_heightmap(chunk);
        let svo = self.generate_chunk_svo(chunk, &heightmap);
        self.loaded.insert(chunk, svo);
    }

    pub fn generate_chunk_gpu(&mut self, chunk: Vector3<i64>) {
        let mesh = self.generate_chunk_mesh(chunk, &self.loaded[&chunk]);
        debug!(
            "generated chunk, \x1B[1mid\x1B[0m: {},{},{}, \x1B[1mvertices\x1B[0m: {}",
            chunk.x,
            chunk.y,
            chunk.z,
            mesh.vertices.len()
        );
        if mesh.vertices.is_empty() {
            return;
        }
        let old_vertex_count = self.vertices.load(Ordering::SeqCst) as usize;
        let new_vertex_count = old_vertex_count + mesh.vertices.len();
        let memory = &mut self.buffer[old_vertex_count..new_vertex_count];
        MaybeUninit::write_slice(memory, &mesh.vertices);
        self.vertices
            .store(new_vertex_count as u64, Ordering::SeqCst);
    }

    pub fn generate_chunk_heightmap(&self, chunk: Vector3<i64>) -> DMatrix<i64> {
        let chunk_coordinates = chunk.xy() * self.chunk_size as i64;
        let mut heightmap = DMatrix::from_element(self.chunk_size, self.chunk_size, 0);
        for x in 0..self.chunk_size {
            for y in 0..self.chunk_size {
                let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
                let noise_position = column_coordinates.cast::<f64>() / 128.;
                let noise_arguments: [f64; 2] = noise_position.into();
                let raw_noise = self.heightmap_noise.get(noise_arguments);
                let scaled_noise = raw_noise * 32.;
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
        assert_eq!(heightmap.nrows(), self.chunk_size);
        assert_eq!(heightmap.ncols(), self.chunk_size);
        svo_from_heightmap_impl(
            0,
            0,
            chunk.z * self.chunk_size as i64,
            self.chunk_size,
            heightmap,
        )
    }

    pub fn generate_chunk_mesh(&self, chunk: Vector3<i64>, voxels: &SparseOctree) -> MeshData {
        let cube = BinaryCube::new_at_zero(self.chunk_size);
        let mut vertices = Vec::new();
        self.generate_chunk_mesh_impl(cube, voxels, voxels, chunk, &mut vertices);
        for vertex in &mut vertices {
            vertex.position += (chunk * DEFAULT_VOXEL_CHUNK_SIZE as i64).cast::<f32>();
        }
        MeshData { vertices }
    }

    fn generate_chunk_mesh_impl(
        &self,
        cube: BinaryCube,
        cube_voxels: &SparseOctree,
        root_voxels: &SparseOctree,
        chunk: Vector3<i64>,
        vertices: &mut Vec<Vertex>,
    ) {
        if cube.is_single_voxel() {
            self.generate_chunk_mesh_voxel(cube.position, root_voxels, chunk, vertices);
            return;
        }

        match cube_voxels {
            SparseOctree::Uniform { .. } => {
                for side_voxel in cube.side_voxels() {
                    self.generate_chunk_mesh_voxel(side_voxel, root_voxels, chunk, vertices);
                }
            }
            SparseOctree::Mixed { children } => {
                for (sub_cube, child) in cube.subdivide().zip(children.iter()) {
                    self.generate_chunk_mesh_impl(sub_cube, child, root_voxels, chunk, vertices);
                }
            }
        };
    }

    fn generate_chunk_mesh_voxel(
        &self,
        position: Vector3<i64>,
        root_voxels: &SparseOctree,
        chunk: Vector3<i64>,
        vertices: &mut Vec<Vertex>,
    ) {
        for direction in DIRECTIONS {
            let side = self.generate_chunk_mesh_side(position, direction, root_voxels, chunk);
            if let Some(side) = side {
                vertices.extend_from_slice(&side);
            }
        }
    }

    fn generate_chunk_mesh_side(
        &self,
        position: Vector3<i64>,
        normal: Vector3<i64>,
        voxels: &SparseOctree,
        chunk: Vector3<i64>,
    ) -> Option<[Vertex; 6]> {
        if !voxels.at(position, self.chunk_size as i64) {
            return None;
        }
        let neighbour = position + normal;
        let to_out_of_bounds = (neighbour.x < 0 || neighbour.x >= self.chunk_size as i64)
            || (neighbour.y < 0 || neighbour.y >= self.chunk_size as i64)
            || (neighbour.z < 0 || neighbour.z >= self.chunk_size as i64);
        if to_out_of_bounds {
            let neighbour_voxels = &self.loaded[&(chunk + normal)];
            let neighbour_in_chunk = Vector3::new(
                (neighbour.x + self.chunk_size as i64) % self.chunk_size as i64,
                (neighbour.y + self.chunk_size as i64) % self.chunk_size as i64,
                (neighbour.z + self.chunk_size as i64) % self.chunk_size as i64,
            );
            if neighbour_voxels.at(neighbour_in_chunk, self.chunk_size as i64) {
                return None;
            }
        } else {
            if voxels.at(neighbour, self.chunk_size as i64) {
                return None;
            }
        }
        let rot1 = Vector3::new(normal.z.abs(), normal.x.abs(), normal.y.abs());
        let rot2 = Vector3::new(normal.y.abs(), normal.z.abs(), normal.x.abs());
        let base = if normal.x + normal.y + normal.z > 0 {
            position + normal
        } else {
            position
        };
        let (rot1, rot2) = if normal == rot1.cross(&rot2) {
            (rot1, rot2)
        } else {
            (rot2, rot1)
        };
        let v1 = Vertex {
            position: base.cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v2 = Vertex {
            position: (base + rot1).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v3 = Vertex {
            position: (base + rot2).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v4 = Vertex {
            position: (base + rot1 + rot2).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        Some([v1, v2, v3, v2, v4, v3])
    }

    pub fn shared(&self) -> Arc<AtomicU64> {
        self.vertices.clone()
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
        let is_stone = heightmap[(x, y)] > z;
        for ly in y..y + n {
            for lx in x..x + n {
                let height = heightmap[(lx, ly)];
                if height > z && height < z + n as i64 {
                    break 'check_all_same;
                }
                if height <= z && is_stone {
                    break 'check_all_same;
                }
                if height >= z + n as i64 && !is_stone {
                    break 'check_all_same;
                }
            }
        }
        return SparseOctree::Uniform {
            kind: if is_stone {
                VoxelKind::Stone
            } else {
                VoxelKind::Air
            },
        };
    }
    let mut children = Vec::new();
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                children.push(svo_from_heightmap_impl(
                    x + dx * n / 2,
                    y + dy * n / 2,
                    z + dz * n as i64 / 2,
                    n / 2,
                    heightmap,
                ));
            }
        }
    }
    let children = std::array::from_fn(|i| Box::new(children[i].clone()));
    SparseOctree::Mixed { children }
}
