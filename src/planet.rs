use crate::interface::Editable;
use crate::model::Model;
use crate::renderer::vertex::Vertex;
use imgui::Ui;
use nalgebra::{Vector2, Vector3};
use noise::NoiseFn;

#[derive(Clone, PartialEq)]
pub struct Parameters {
    pub resolution: usize,
    pub radius: f32,
    pub noise_magnitude: f32,
    pub noise_scale: f32,
}

struct Side {
    base: Vector3<f32>,
    dx: Vector3<f32>,
    dy: Vector3<f32>,
}

const SIDES: [Side; 6] = [
    Side {
        base: Vector3::new(-1., 1., -1.),
        dx: Vector3::new(0., -1., 0.),
        dy: Vector3::new(0., 0., 1.),
    },
    Side {
        base: Vector3::new(-1., -1., -1.),
        dx: Vector3::new(1., 0., 0.),
        dy: Vector3::new(0., 0., 1.),
    },
    Side {
        base: Vector3::new(1., -1., -1.),
        dx: Vector3::new(0., 1., 0.),
        dy: Vector3::new(0., 0., 1.),
    },
    Side {
        base: Vector3::new(1., 1., -1.),
        dx: Vector3::new(-1., 0., 0.),
        dy: Vector3::new(0., 0., 1.),
    },
    Side {
        base: Vector3::new(-1., 1., -1.),
        dx: Vector3::new(1., 0., 0.),
        dy: Vector3::new(0., -1., 0.),
    },
    Side {
        base: Vector3::new(-1., 1., 1.),
        dx: Vector3::new(0., -1., 0.),
        dy: Vector3::new(1., 0., 0.),
    },
];

impl Editable for Parameters {
    fn name(&self) -> &str {
        "Planet generation"
    }

    fn widget(&mut self, ui: &Ui) {
        ui.slider("Resolution", 1, 10000, &mut self.resolution);
        ui.slider("Radius", 10., 200., &mut self.radius);
        ui.slider("Noise magnitude", 0., 100., &mut self.noise_magnitude);
        ui.slider("Noise scale", 0., 64., &mut self.noise_scale);
    }
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            resolution: 400,
            radius: 100.,
            noise_magnitude: 20.,
            noise_scale: 16.,
        }
    }
}

pub fn generate_planet(parameters: &Parameters) -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let noise = noise::Perlin::new(907);
    for (k, side) in SIDES.iter().enumerate() {
        for i in 0..parameters.resolution + 1 {
            for j in 0..parameters.resolution + 1 {
                let direction = (side.base
                    + i as f32 * (2. * side.dx) / parameters.resolution as f32
                    + j as f32 * (2. * side.dy) / parameters.resolution as f32)
                    .normalize();
                let is_on_edge =
                    i == 0 || i == parameters.resolution || j == 0 || j == parameters.resolution;
                let position = direction
                    * (parameters.radius
                        + if is_on_edge {
                            0.
                        } else {
                            let x =
                                parameters.noise_scale * i as f32 / parameters.resolution as f32;
                            let y =
                                parameters.noise_scale * j as f32 / parameters.resolution as f32;
                            parameters.noise_magnitude * noise.get([x as f64, y as f64]) as f32
                        });
                let normal = direction;
                let tex = Vector2::zeros();
                let vertex = Vertex {
                    position,
                    normal,
                    tex,
                };
                vertices.push(vertex);
            }
        }
        for i in 0..parameters.resolution as u32 {
            for j in 0..parameters.resolution as u32 {
                let side_offset = k as u32
                    * (parameters.resolution as u32 + 1)
                    * (parameters.resolution as u32 + 1);
                indices.push(side_offset + i * (parameters.resolution as u32 + 1) + j);
                indices.push(side_offset + (i + 1) * (parameters.resolution as u32 + 1) + j);
                indices.push(side_offset + i * (parameters.resolution as u32 + 1) + j + 1);

                indices.push(side_offset + i * (parameters.resolution as u32 + 1) + j + 1);
                indices.push(side_offset + (i + 1) * (parameters.resolution as u32 + 1) + j);
                indices.push(side_offset + (i + 1) * (parameters.resolution as u32 + 1) + j + 1);
            }
        }
    }
    Model {
        vertices,
        indices,
        texture_path: "assets/cube.png",
    }
}
