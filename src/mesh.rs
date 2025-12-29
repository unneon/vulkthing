use crate::gpu::ClassicVertex;
use nalgebra::Vector3;
use std::io::BufReader;
use tobj::{LoadError, LoadOptions};

#[derive(Clone, Debug)]
pub struct MeshData<V> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}

pub fn load_mesh(obj: &str) -> MeshData<ClassicVertex> {
    let load_options = LoadOptions {
        // Faces can sometimes be given as arbitrary (convex?) polygons, but we only render
        // triangles so let's get the loader to split them up for us.
        triangulate: true,
        // Some models use separate sets of indices for vertices and texture coordinates. The Rust
        // version of the tutorial didn't include this, but it probably should have.
        single_index: true,
        ..Default::default()
    };
    let mut obj_reader = BufReader::new(obj.as_bytes());
    let meshes = tobj::load_obj_buf(&mut obj_reader, &load_options, |_| {
        Err(LoadError::OpenFileFailed)
    })
    .unwrap()
    .0;
    flatten_meshes(&meshes)
}

fn flatten_meshes(models: &[tobj::Model]) -> MeshData<ClassicVertex> {
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
            let vertex = ClassicVertex { position, normal };
            vertices.push(vertex);
        }
    }
    let indices = (0..vertices.len() as u32).collect();
    for v123 in vertices.chunks_mut(3) {
        let [v1, v2, v3] = v123 else { unreachable!() };
        let normal = (v2.position - v1.position)
            .cross(&(v3.position - v1.position))
            .normalize();
        v1.normal = normal;
        v2.normal = normal;
        v3.normal = normal;
    }
    MeshData { vertices, indices }
}
