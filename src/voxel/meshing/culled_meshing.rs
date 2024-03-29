use crate::voxel::binary_cube::BinaryCube;
use crate::voxel::local_mesh::{LocalFace, LocalMesh, LocalVertex};
use crate::voxel::meshing::MeshingAlgorithm;
use crate::voxel::neighbourhood::Neighbourhood;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::DIRECTIONS;
use nalgebra::Vector3;

pub struct CulledMeshing;

struct State<'a> {
    chunk_size: usize,
    svos: &'a Neighbourhood<'a>,
    vertices: Vec<LocalVertex>,
    faces: Vec<LocalFace>,
}

impl MeshingAlgorithm for CulledMeshing {
    fn mesh(svos: &Neighbourhood, chunk_size: usize) -> LocalMesh {
        let cube = BinaryCube::new_at_zero(chunk_size);
        let chunk = svos.chunk();
        let mut state = State {
            chunk_size,
            svos,
            vertices: Vec::new(),
            faces: Vec::new(),
        };
        state.mesh_cube(cube, chunk);
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
        let material = self.svos.chunk().at(position, chunk_size);
        if material.is_air() {
            return;
        }
        let normal = DIRECTIONS[normal_index];
        if !self.svos.at(position + normal).is_air() {
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
        let v1 = self.make_vertex(base, normal);
        let v2 = self.make_vertex(base + rot1, normal);
        let v3 = self.make_vertex(base + rot2, normal);
        let v4 = self.make_vertex(base + rot1 + rot2, normal);
        let indices = if v1.ambient_occlusion + v4.ambient_occlusion
            >= v2.ambient_occlusion + v3.ambient_occlusion
        {
            [i1, i2, i3, i4]
        } else {
            [i2, i4, i1, i3]
        };
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v4);
        self.faces.push(LocalFace {
            indices,
            normal_index: normal_index as u8,
            material,
        });
    }

    fn make_vertex(&self, position: Vector3<i64>, normal: Vector3<i64>) -> LocalVertex {
        // TODO: Not sure if correct for other normals than 0,0,1.
        let occluder_base = if normal.sum() < 0 {
            position + normal
        } else {
            position
        };
        let u = normal.zxy().abs();
        let v = normal.yzx().abs();
        let side1 = !self.svos.at(occluder_base - u).is_air();
        let side2 = !self.svos.at(occluder_base - v).is_air();
        let corner = (!self.svos.at(occluder_base).is_air())
            || (!self.svos.at(occluder_base - u - v).is_air());
        let ambient_occlusion = if side1 && side2 {
            3
        } else {
            (if side1 { 1 } else { 0 }) + (if side2 { 1 } else { 0 }) + (if corner { 1 } else { 0 })
        };
        LocalVertex {
            position: position.try_cast::<u8>().unwrap(),
            ambient_occlusion,
        }
    }
}
