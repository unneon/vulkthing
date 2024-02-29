use crate::mesh::MeshData;
use crate::voxel::vertex::VoxelVertex;
use crate::voxel::DIRECTIONS;
use meshopt::{build_meshlets, typed_to_bytes, VertexDataAdapter};

#[derive(Debug)]
pub struct VoxelMesh {
    pub meshlets: Vec<VoxelMeshlet>,
    pub vertices: Vec<VoxelVertex>,
    pub triangles: Vec<VoxelTriangle>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelTriangle {
    index0: u8,
    index1: u8,
    index2: u8,
    data: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct VoxelMeshlet {
    pub vertex_offset: u32,
    pub vertex_count: u32,
    pub triangle_offset: u32,
    pub triangle_count: u32,
}

impl VoxelTriangle {
    fn new(indices: [u8; 3], normal: u8, material: u8) -> VoxelTriangle {
        assert!(normal < 1 << 3);
        assert!(material < 1 << 5);
        VoxelTriangle {
            index0: indices[0],
            index1: indices[1],
            index2: indices[2],
            data: normal | (material << 3),
        }
    }
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
    // The naming of output fields is confusing, the triangles array is an unpacked index buffer for triangles. We
    // change the name to indices, to stuff like "index count" is valid and not even worse.
    let mut meshlets = Vec::new();
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    for meshlet in raw_meshlets.iter() {
        let vertex_offset = vertices.len() as u32;
        let triangle_offset = triangles.len() as u32;
        for &vertex in meshlet.vertices {
            vertices.push(mesh.vertices[vertex as usize]);
        }
        for &indices in meshlet.triangles.array_chunks() {
            let v0 = mesh.vertices[meshlet.vertices[indices[0] as usize] as usize];
            let v1 = mesh.vertices[meshlet.vertices[indices[1] as usize] as usize];
            let v2 = mesh.vertices[meshlet.vertices[indices[2] as usize] as usize];
            let normal_approx = (v1.position - v0.position)
                .cross(&(v2.position - v0.position))
                .normalize();
            let normal_index = DIRECTIONS
                .iter()
                .enumerate()
                .find(|(_, direction)| (direction.cast::<f32>() - normal_approx).norm() < 0.001)
                .unwrap()
                .0;
            triangles.push(VoxelTriangle::new(indices, normal_index as u8, 3));
        }
        meshlets.push(VoxelMeshlet {
            vertex_offset,
            vertex_count: meshlet.vertices.len() as u32,
            triangle_offset,
            triangle_count: meshlet.triangles.len() as u32 / 3,
        });
    }
    VoxelMesh {
        meshlets,
        vertices,
        triangles,
    }
}
