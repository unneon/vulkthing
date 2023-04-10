use crate::renderer::vertex::Vertex;
use log::debug;
use nalgebra_glm as glm;
use tobj::LoadOptions;

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub texture_path: &'static str,
}

pub fn load_model(obj_path: &str, texture_path: &'static str) -> Model {
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
        "model OBJ loaded, \x1B[1mpath\x1B[0m: {obj_path}, \x1B[1mvertices\x1B[0m: {}",
        vertices.len()
    );
    Model {
        vertices,
        indices,
        texture_path,
    }
}

fn flatten_models(models: &[tobj::Model]) -> (Vec<Vertex>, Vec<u32>) {
    // OBJ format supports quite complex models with many materials and meshes, but temporarily
    // let's just throw all of it into a single vertex buffer.
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for model in models {
        for index in &model.mesh.indices {
            // Position vectors and texture coordinate vectors are stored as unpacked arrays of
            // floats.
            let offset_pos = (3 * *index) as usize;
            let offset_tex = (2 * *index) as usize;
            let position = glm::vec3(
                model.mesh.positions[offset_pos],
                model.mesh.positions[offset_pos + 1],
                model.mesh.positions[offset_pos + 2],
            );
            // Will be computed from triangle positions later.
            let normal = glm::zero();
            // Coordinate system in OBJ assumes that 0 is the bottom of the image, but Vulkan uses
            // an orientation where 0 is the top of the image.
            let tex = if model.mesh.texcoords.is_empty() {
                glm::vec2(0., 0.)
            } else {
                glm::vec2(
                    model.mesh.texcoords[offset_tex],
                    1.0 - model.mesh.texcoords[offset_tex + 1],
                )
            };
            let vertex = Vertex {
                position,
                normal,
                tex,
            };
            let index = vertices.len();
            vertices.push(vertex);
            indices.push(index as u32);
        }
    }
    for [v1, v2, v3] in vertices.array_chunks_mut() {
        let normal =
            glm::cross(&(v2.position - v1.position), &(v3.position - v1.position)).normalize();
        v1.normal = normal;
        v2.normal = normal;
        v3.normal = normal;
    }
    (vertices, indices)
}

fn scale_mesh(vertices: &mut [Vertex]) {
    let mut min = glm::vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = glm::vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for vertex in vertices.iter_mut() {
        min = glm::min2(&min, &vertex.position);
        max = glm::max2(&max, &vertex.position);
    }
    for vertex in vertices.iter_mut() {
        vertex.position =
            ((vertex.position - min).component_div(&(max - min)) * 2.).add_scalar(-1.);
    }
}
