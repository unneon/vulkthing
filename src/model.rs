use nalgebra_glm as glm;
use std::collections::{hash_map, HashMap};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub tex: glm::Vec2,
}

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn load_model() -> Model {
    let models = tobj::load_obj("assets/viking-room.obj", &Default::default())
        .unwrap()
        .0;
    // OBJ format supports quite complex models with many materials and meshes, but temporarily
    // let's just throw all of it into a single vertex buffer.
    let mut unique_vertices = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for model in models {
        for index in model.mesh.indices {
            // Position vectors and texture coordinate vectors are stored as unpacked arrays of
            // floats.
            let offset_pos = (3 * index) as usize;
            let offset_tex = (2 * index) as usize;
            let position = glm::vec3(
                model.mesh.positions[offset_pos],
                model.mesh.positions[offset_pos + 1],
                model.mesh.positions[offset_pos + 2],
            );
            // Coordinate system in OBJ assumes that 0 is the bottom of the image, but Vulkan uses
            // an orientation where 0 is the top of the image.
            let tex = glm::vec2(
                model.mesh.texcoords[offset_tex],
                1.0 - model.mesh.texcoords[offset_tex + 1],
            );
            let vertex = Vertex { position, tex };
            let index = match unique_vertices.entry(vertex) {
                hash_map::Entry::Occupied(e) => *e.get(),
                hash_map::Entry::Vacant(e) => {
                    let index = vertices.len();
                    e.insert(index);
                    vertices.push(vertex);
                    index
                }
            };
            indices.push(index as u32);
        }
    }
    Model { vertices, indices }
}
