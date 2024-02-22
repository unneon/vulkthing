use crate::renderer::vertex::{Vertex, VoxelVertex};
use log::debug;
use nalgebra::Vector3;
use std::collections::{hash_map, HashMap};
use std::hash::Hash;
use std::io::Write;
use tobj::LoadOptions;

#[derive(Clone, Debug)]
pub struct MeshData<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

impl<V: Clone + Eq + Hash + PartialEq> MeshData<V> {
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
        let mut indices = Vec::new();
        for index in &self.indices {
            indices.push(mapping[&self.vertices[*index as usize]]);
        }
        MeshData { vertices, indices }
    }
}

impl MeshData<VoxelVertex> {
    pub fn write_obj(&self, mut f: impl Write) -> std::io::Result<()> {
        for vertex in &self.vertices {
            writeln!(
                f,
                "v {} {} {}",
                vertex.position.x, vertex.position.y, vertex.position.z
            )?;
        }
        for [i1, i2, i3] in self.indices.array_chunks() {
            writeln!(f, "f {} {} {}", i1 + 1, i2 + 1, i3 + 1)?;
        }
        Ok(())
    }
}

pub fn load_mesh(obj_path: &str) -> MeshData<Vertex> {
    let load_options = LoadOptions {
        // Faces can sometimes be given as arbitrary (convex?) polygons, but we only render
        // triangles so let's get the loader to split them up for us.
        triangulate: true,
        // Some models use separate sets of indices for vertices and texture coordinates. The Rust
        // version of the tutorial didn't include this, but it probably should have.
        single_index: true,
        ..Default::default()
    };
    let meshes = tobj::load_obj(obj_path, &load_options).unwrap().0;
    let mesh = flatten_meshes(&meshes);
    debug!(
        "mesh OBJ loaded, \x1B[1mfile\x1B[0m: {obj_path}, \x1B[1mvertices\x1B[0m: {}",
        mesh.vertices.len()
    );
    mesh
}

fn flatten_meshes(models: &[tobj::Model]) -> MeshData<Vertex> {
    // OBJ format supports quite complex meshes with many materials and meshes, but temporarily
    // let's just throw all of it into a single vertex buffer.
    let mut vertices = Vec::new();
    for model in models {
        for index in &model.mesh.indices {
            // Position vectors are stored as unpacked arrays of floats.
            let offset = (3 * *index) as usize;
            let position = Vector3::new(
                model.mesh.positions[offset],
                model.mesh.positions[offset + 1],
                model.mesh.positions[offset + 2],
            );
            // Will be computed from triangle positions later.
            let normal = Vector3::zeros();
            let vertex = Vertex { position, normal };
            vertices.push(vertex);
        }
    }
    let indices = (0..vertices.len() as u32).collect();
    for [v1, v2, v3] in vertices.array_chunks_mut() {
        let normal = (v2.position - v1.position)
            .cross(&(v3.position - v1.position))
            .normalize();
        v1.normal = normal;
        v2.normal = normal;
        v3.normal = normal;
    }
    MeshData { vertices, indices }
}
