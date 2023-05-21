use crate::renderer::vertex::Vertex;
use log::debug;
use nalgebra::Vector3;
use tobj::LoadOptions;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn load_model(obj_path: &str) -> Model {
    let load_options = LoadOptions {
        // Faces can sometimes be given as arbitrary (convex?) polygons, but we only render
        // triangles so let's get the loader to split them up for us.
        triangulate: true,
        // Some models use separate sets of indices for vertices and texture coordinates. The Rust
        // version of the tutorial didn't include this, but it probably should have.
        single_index: true,
        ..Default::default()
    };
    let models = tobj::load_obj(obj_path, &load_options).unwrap().0;
    let (mut vertices, indices) = flatten_models(&models);
    scale_mesh(&mut vertices);
    debug!(
        "model OBJ loaded, \x1B[1mfile\x1B[0m: {obj_path}, \x1B[1mvertices\x1B[0m: {}",
        vertices.len()
    );
    Model { vertices, indices }
}

fn flatten_models(models: &[tobj::Model]) -> (Vec<Vertex>, Vec<u32>) {
    // OBJ format supports quite complex models with many materials and meshes, but temporarily
    // let's just throw all of it into a single vertex buffer.
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
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
            let index = vertices.len();
            vertices.push(vertex);
            indices.push(index as u32);
        }
    }
    for [v1, v2, v3] in vertices.array_chunks_mut() {
        let normal = (v2.position - v1.position)
            .cross(&(v3.position - v1.position))
            .normalize();
        v1.normal = normal;
        v2.normal = normal;
        v3.normal = normal;
    }
    (vertices, indices)
}

fn scale_mesh(vertices: &mut [Vertex]) {
    let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for vertex in vertices.iter_mut() {
        min = min.inf(&vertex.position);
        max = max.sup(&vertex.position);
    }
    for vertex in vertices.iter_mut() {
        vertex.position =
            ((vertex.position - min).component_div(&(max - min)) * 2.).add_scalar(-1.);
    }
}
