use crate::mesh::MeshData;
use crate::voxel::vertex::VoxelVertex;
use meshopt::{build_meshlets, typed_to_bytes, VertexDataAdapter};

#[derive(Debug)]
pub struct VoxelMesh {
    pub meshlets: Vec<VoxelMeshlet>,
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelMeshlet {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

pub fn from_unclustered_mesh(mesh: &MeshData<VoxelVertex>) -> VoxelMesh {
    if mesh.indices.is_empty() {
        return VoxelMesh {
            meshlets: Vec::new(),
            vertices: Vec::new(),
            indices: Vec::new(),
        };
    }
    let vertices = VertexDataAdapter::new(typed_to_bytes(&mesh.vertices), 16, 0).unwrap();
    let raw_meshlets = build_meshlets(&mesh.indices, &vertices, 128, 256, 0.);
    // The naming of output fields is confusing, the triangles array is an unpacked index buffer for triangles. We
    // change the name to indices, to stuff like "index count" is valid and not even worse.
    let mut meshlets = Vec::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for meshlet in raw_meshlets.iter() {
        let vertex_offset = vertices.len() as u32;
        let index_offset = indices.len() as u32;
        for &vertex in meshlet.vertices {
            vertices.push(mesh.vertices[vertex as usize]);
        }
        for &triangle_index in meshlet.triangles {
            indices.push(triangle_index);
        }
        meshlets.push(VoxelMeshlet {
            vertex_offset,
            vertex_count: meshlet.vertices.len() as u32,
            index_offset,
            index_count: meshlet.triangles.len() as u32,
        });
    }
    VoxelMesh {
        meshlets,
        vertices,
        indices,
    }
}
