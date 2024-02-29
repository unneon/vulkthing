use crate::voxel::material::Material;
use nalgebra::Vector3;
use std::collections::{hash_map, HashMap};

pub struct LocalMesh {
    pub vertices: Vec<LocalVertex>,
    pub faces: Vec<LocalFace>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LocalVertex {
    pub position: Vector3<u16>,
}

#[derive(Clone)]
pub struct LocalFace {
    pub indices: [u32; 4],
    pub normal_index: u8,
    pub material: Material,
}

impl LocalMesh {
    pub fn new_empty() -> LocalMesh {
        LocalMesh {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn remove_duplicate_vertices(self) -> Self {
        let mut mapping = HashMap::new();
        let mut counter = 0;
        let mut vertices = Vec::new();
        for vertex in &self.vertices {
            if let hash_map::Entry::Vacant(mapping) = mapping.entry(vertex) {
                mapping.insert(counter);
                counter += 1;
                vertices.push(vertex.clone());
            }
        }
        let mut faces = Vec::new();
        for face in &self.faces {
            faces.push(LocalFace {
                indices: face
                    .indices
                    .map(|index| mapping[&self.vertices[index as usize]]),
                normal_index: face.normal_index,
                material: face.material,
            });
        }
        LocalMesh { vertices, faces }
    }
}
