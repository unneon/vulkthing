use crate::mesh::MeshData;
use crate::renderer::uniform::VoxelMeshlet;
use crate::renderer::vertex::VoxelVertex;
use meshopt::{build_meshlets, typed_to_bytes, VertexDataAdapter};

#[derive(Debug)]
pub struct VoxelMesh {
    pub meshlets: Vec<VoxelMeshlet>,
    pub vertices: Vec<VoxelVertex>,
    pub triangles: Vec<u32>,
}

pub fn from_unclustered_mesh(mesh: &MeshData<VoxelVertex>) -> VoxelMesh {
    if mesh.indices.is_empty() {
        return VoxelMesh {
            meshlets: Vec::new(),
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
    }
    let vertices = VertexDataAdapter::new(typed_to_bytes(&mesh.vertices), 16, 0).unwrap();
    let raw_meshlets = build_meshlets(&mesh.indices, &vertices, 128, 256, 0.);
    let mut meshlets = Vec::new();
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for meshlet in raw_meshlets.iter() {
        let vertex_offset = vertices.len() as u32;
        let triangle_offset = triangles.len() as u32;
        for &vertex in meshlet.vertices {
            vertices.push(mesh.vertices[vertex as usize]);
        }
        for &triangle_index in meshlet.triangles {
            triangles.push(triangle_index as u32);
        }
        meshlets.push(VoxelMeshlet {
            vertex_offset,
            vertex_count: meshlet.vertices.len() as u32,
            triangle_offset,
            triangle_count: meshlet.triangles.len() as u32,
        });
    }
    VoxelMesh {
        meshlets,
        vertices,
        triangles,
    }
}
