use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use crate::voxels::binary_cube::BinaryCube;
use crate::voxels::sparse_octree::SparseOctree;
use crate::voxels::{MeshingAlgorithm, VoxelKind, DIRECTIONS};
use nalgebra::Vector3;

pub struct CulledMeshing;

struct State<'a> {
    chunk_size: usize,
    chunk_svo: &'a SparseOctree,
    neighbour_svos: [&'a SparseOctree; 6],
    vertices: Vec<Vertex>,
}

impl MeshingAlgorithm for CulledMeshing {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> MeshData {
        let cube = BinaryCube::new_at_zero(chunk_size);
        let mut state = State {
            chunk_size,
            chunk_svo,
            neighbour_svos,
            vertices: Vec::new(),
        };
        state.mesh_cube(cube, chunk_svo);
        MeshData {
            vertices: state.vertices,
        }
    }
}

impl State<'_> {
    fn mesh_cube(&mut self, cube: BinaryCube, cube_svo: &SparseOctree) {
        if cube.is_single_voxel() {
            self.mesh_voxel(cube.position);
            return;
        }
        match cube_svo {
            SparseOctree::Uniform { .. } => {
                for side_voxel in cube.side_voxels() {
                    self.mesh_voxel(side_voxel);
                }
            }
            SparseOctree::Mixed { children } => {
                for (child, child_svo) in cube.subdivide().zip(children.iter()) {
                    self.mesh_cube(child, child_svo);
                }
            }
        };
    }

    fn mesh_voxel(&mut self, position: Vector3<i64>) {
        for direction_index in 0..DIRECTIONS.len() {
            let side = self.mesh_voxel_side(position, direction_index);
            if let Some(side) = side {
                self.vertices.extend_from_slice(&side);
            }
        }
    }

    fn mesh_voxel_side(
        &mut self,
        position: Vector3<i64>,
        normal_index: usize,
    ) -> Option<[Vertex; 6]> {
        let chunk_size = self.chunk_size as i64;
        if self.chunk_svo.at(position, chunk_size) == VoxelKind::Air {
            return None;
        }
        let normal = DIRECTIONS[normal_index];
        let neighbour_in_chunk = position + normal;
        let is_neighbour_outside_chunk = neighbour_in_chunk.x < 0
            || neighbour_in_chunk.x >= chunk_size
            || neighbour_in_chunk.y < 0
            || neighbour_in_chunk.y >= chunk_size
            || neighbour_in_chunk.z < 0
            || neighbour_in_chunk.z >= chunk_size;
        if is_neighbour_outside_chunk {
            let neighbour_svo = &self.neighbour_svos[normal_index];
            let neighbour_in_neighbour = Vector3::new(
                (neighbour_in_chunk.x + chunk_size) % chunk_size,
                (neighbour_in_chunk.y + chunk_size) % chunk_size,
                (neighbour_in_chunk.z + chunk_size) % chunk_size,
            );
            if neighbour_svo.at(neighbour_in_neighbour, chunk_size) == VoxelKind::Stone {
                return None;
            }
        } else if self.chunk_svo.at(neighbour_in_chunk, chunk_size) == VoxelKind::Stone {
            return None;
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
}
