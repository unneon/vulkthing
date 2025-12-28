use crate::gpu::{VoxelMeshlet, VoxelTriangle, VoxelVertex};
use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::material::Material;
use crate::voxel::sparse_octree::SparseOctree;
use meshopt::{build_meshlets, typed_to_bytes, VertexDataAdapter};
use nalgebra::Vector3;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VoxelMesh {
    pub meshlets: Vec<VoxelMeshlet>,
    pub vertices: Vec<VoxelVertex>,
    pub triangles: Vec<VoxelTriangle>,
    pub octree: SparseOctree,
    pub chunk: Vector3<i64>,
}

// Data format expected by the meshoptimizer library. I'll be writing my own meshlet construction
// algorithm later anyway, so the inefficiency doesn't matter for now.
struct MeshoptVertex {
    #[allow(dead_code)]
    position: Vector3<f32>,
}

impl VoxelVertex {
    fn new(position: Vector3<u8>, ambient_occlusion: u8) -> VoxelVertex {
        assert!(ambient_occlusion < 4);
        VoxelVertex {
            position,
            data: ambient_occlusion,
        }
    }
}

impl VoxelTriangle {
    fn new(indices: [u8; 3], normal: u8, material: Material) -> VoxelTriangle {
        assert!(normal < 6);
        VoxelTriangle {
            indices: indices.into(),
            data: normal | ((material as u8) << 3),
        }
    }
}

pub fn from_unclustered_mesh(
    mesh: &LocalMesh,
    octree: &SparseOctree,
    chunk: Vector3<i64>,
) -> VoxelMesh {
    if mesh.faces.is_empty() {
        return VoxelMesh {
            meshlets: Vec::new(),
            vertices: Vec::new(),
            triangles: Vec::new(),
            octree: octree.clone(),
            chunk,
        };
    }
    let mut triangle_to_face = HashMap::new();
    for (face_index, face) in mesh.faces.iter().enumerate() {
        triangle_to_face.insert(
            [face.indices[0], face.indices[1], face.indices[2]],
            face_index,
        );
        triangle_to_face.insert(
            [face.indices[1], face.indices[3], face.indices[2]],
            face_index,
        );
    }
    let raw_meshlets = build_raw_meshlets(mesh);
    let mut meshlets = Vec::new();
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for meshlet in raw_meshlets.iter() {
        let vertex_offset = vertices.len() as u32;
        let triangle_offset = triangles.len() as u32;
        let mut min_coords = Vector3::from_element(255);
        let mut max_coords = Vector3::from_element(0);
        for &vertex in meshlet.vertices {
            let vertex = &mesh.vertices[vertex as usize];
            vertices.push(VoxelVertex::new(vertex.position, vertex.ambient_occlusion));
            min_coords.x = min_coords.x.min(vertex.position.x);
            min_coords.y = min_coords.y.min(vertex.position.y);
            min_coords.z = min_coords.z.min(vertex.position.z);
            max_coords.x = max_coords.x.max(vertex.position.x);
            max_coords.y = max_coords.y.max(vertex.position.y);
            max_coords.z = max_coords.z.max(vertex.position.z);
        }
        let bound_base = min_coords;
        let bound_size = max_coords - min_coords;
        for mi012 in meshlet.triangles.chunks(3) {
            let &[mi0, mi1, mi2] = mi012 else {
                unreachable!()
            };
            let i0 = meshlet.vertices[mi0 as usize];
            let i1 = meshlet.vertices[mi1 as usize];
            let i2 = meshlet.vertices[mi2 as usize];
            let face_index = triangle_to_face[&[i0, i1, i2]];
            let face = &mesh.faces[face_index];
            triangles.push(VoxelTriangle::new(
                [mi0, mi1, mi2],
                face.normal_index,
                face.material,
            ));
        }
        meshlets.push(VoxelMeshlet {
            vertex_offset,
            vertex_count: meshlet.vertices.len() as u32,
            triangle_offset,
            triangle_count: meshlet.triangles.len() as u32 / 3,
            chunk: Vector3::zeros(),
            bound_base,
            bound_size,
        });
    }
    VoxelMesh {
        meshlets,
        vertices,
        triangles,
        octree: octree.clone(),
        chunk,
    }
}

fn build_raw_meshlets(mesh: &LocalMesh) -> meshopt::Meshlets {
    let mut meshopt_indices = Vec::new();
    for face in &mesh.faces {
        meshopt_indices.extend_from_slice(&[
            face.indices[0],
            face.indices[1],
            face.indices[2],
            face.indices[1],
            face.indices[3],
            face.indices[2],
        ]);
    }
    let mut meshopt_vertices = Vec::new();
    for vertex in &mesh.vertices {
        meshopt_vertices.push(MeshoptVertex {
            position: vertex.position.cast::<f32>(),
        });
    }
    let vertices = VertexDataAdapter::new(
        typed_to_bytes(&meshopt_vertices),
        std::mem::size_of::<MeshoptVertex>(),
        0,
    )
    .unwrap();
    build_meshlets(&meshopt_indices, &vertices, 128, 256, 0.)
}
