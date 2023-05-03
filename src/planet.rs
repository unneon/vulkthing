use crate::model::Model;
use crate::renderer::vertex::Vertex;
use nalgebra::{Vector2, Vector3};
use rand::Rng;

struct Side {
    base: Vector3<f32>,
    dx: Vector3<f32>,
    dy: Vector3<f32>,
}

const SIDES: [Side; 6] = [
    Side {
        base: Vector3::new(-1., 1., -1.),
        dx: Vector3::new(0., -0.1, 0.),
        dy: Vector3::new(0., 0., 0.1),
    },
    Side {
        base: Vector3::new(-1., -1., -1.),
        dx: Vector3::new(0.1, 0., 0.),
        dy: Vector3::new(0., 0., 0.1),
    },
    Side {
        base: Vector3::new(1., -1., -1.),
        dx: Vector3::new(0., 0.1, 0.),
        dy: Vector3::new(0., 0., 0.1),
    },
    Side {
        base: Vector3::new(1., 1., -1.),
        dx: Vector3::new(-0.1, 0., 0.),
        dy: Vector3::new(0., 0., 0.1),
    },
    Side {
        base: Vector3::new(-1., 1., -1.),
        dx: Vector3::new(0.1, 0., 0.),
        dy: Vector3::new(0., -0.1, 0.),
    },
    Side {
        base: Vector3::new(-1., 1., 1.),
        dx: Vector3::new(0., -0.1, 0.),
        dy: Vector3::new(0.1, 0., 0.),
    },
];

const COUNT: u32 = 20;

pub fn generate_planet() -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut rng = rand::thread_rng();
    for (k, side) in SIDES.iter().enumerate() {
        for i in 0..COUNT + 1 {
            for j in 0..COUNT + 1 {
                let sphere = (side.base + i as f32 * side.dx + j as f32 * side.dy).normalize();
                let position = if i == 0 || i == COUNT || j == 0 || j == COUNT {
                    sphere * 100.
                } else {
                    sphere * (100. + 15. * rng.gen::<f32>())
                };
                let normal = sphere;
                let tex = Vector2::zeros();
                let vertex = Vertex {
                    position,
                    normal,
                    tex,
                };
                vertices.push(vertex);
            }
        }
        for i in 0..COUNT {
            for j in 0..COUNT {
                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + i * (COUNT + 1) + j);
                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + (i + 1) * (COUNT + 1) + j);
                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + i * (COUNT + 1) + j + 1);

                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + i * (COUNT + 1) + j + 1);
                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + (i + 1) * (COUNT + 1) + j);
                indices.push(k as u32 * (COUNT + 1) * (COUNT + 1) + (i + 1) * (COUNT + 1) + j + 1);
            }
        }
    }
    Model {
        vertices,
        indices,
        texture_path: "assets/cube.png",
    }
}
