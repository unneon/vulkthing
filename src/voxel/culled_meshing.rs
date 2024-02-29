use crate::voxel::binary_cube::BinaryCube;
use crate::voxel::local_mesh::{LocalFace, LocalMesh, LocalVertex};
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::{MeshingAlgorithm, DIRECTIONS};
use nalgebra::Vector3;

pub struct CulledMeshing;

struct State<'a> {
    chunk_size: usize,
    chunk_svo: &'a SparseOctree,
    neighbour_svos: [&'a SparseOctree; 6],
    vertices: Vec<LocalVertex>,
    faces: Vec<LocalFace>,
}

impl MeshingAlgorithm for CulledMeshing {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> LocalMesh {
        let cube = BinaryCube::new_at_zero(chunk_size);
        let mut state = State {
            chunk_size,
            chunk_svo,
            neighbour_svos,
            vertices: Vec::new(),
            faces: Vec::new(),
        };
        state.mesh_cube(cube, chunk_svo);
        LocalMesh {
            vertices: state.vertices,
            faces: state.faces,
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
            self.mesh_voxel_side(position, direction_index);
        }
    }

    fn mesh_voxel_side(&mut self, position: Vector3<i64>, normal_index: usize) {
        let chunk_size = self.chunk_size as i64;
        let material = self.chunk_svo.at(position, chunk_size);
        if material.is_air() {
            return;
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
            if !neighbour_svo
                .at(neighbour_in_neighbour, chunk_size)
                .is_air()
            {
                return;
            }
        } else if !self.chunk_svo.at(neighbour_in_chunk, chunk_size).is_air() {
            return;
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
        let base_index = self.vertices.len() as u32;
        let i1 = base_index;
        let i2 = base_index + 1;
        let i3 = base_index + 2;
        let i4 = base_index + 3;
        let indices = [i1, i2, i3, i4];
        let v1 = LocalVertex {
            position: base.try_cast::<u16>().unwrap(),
        };
        let v2 = LocalVertex {
            position: (base + rot1).try_cast::<u16>().unwrap(),
        };
        let v3 = LocalVertex {
            position: (base + rot2).try_cast::<u16>().unwrap(),
        };
        let v4 = LocalVertex {
            position: (base + rot1 + rot2).try_cast::<u16>().unwrap(),
        };
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v4);
        self.faces.push(LocalFace {
            indices,
            normal_index: normal_index as u8,
            material: material as u8,
        });
    }
}
