use crate::interface::Editable;
use crate::model::Model;
use crate::renderer::vertex::Vertex;
use imgui::Ui;
use nalgebra::{Vector2, Vector3};
use noise::{
    Checkerboard, Cylinders, NoiseFn, OpenSimplex, Perlin, PerlinSurflet, Simplex, SuperSimplex,
    Value, Worley,
};

#[derive(Clone, PartialEq)]
pub struct Parameters {
    resolution: usize,
    radius: f32,
    noise_type: usize,
    noise_magnitude: f32,
    noise_scale: f32,
    noise_layers: usize,
}

enum NoiseType {
    Checkerboard,
    Cylinders,
    OpenSimplex,
    Perlin,
    PerlinSurflet,
    Simplex,
    SuperSimplex,
    Value,
    Worley,
}

struct Side {
    base: Vector3<f32>,
    dx: Vector3<f32>,
    dy: Vector3<f32>,
}

const NOISE_TYPES: &[NoiseType] = &[
    NoiseType::Checkerboard,
    NoiseType::Cylinders,
    NoiseType::OpenSimplex,
    NoiseType::Perlin,
    NoiseType::PerlinSurflet,
    NoiseType::Simplex,
    NoiseType::SuperSimplex,
    NoiseType::Value,
    NoiseType::Worley,
];

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

impl NoiseType {
    fn name(&self) -> &'static str {
        match self {
            NoiseType::Checkerboard => "Checkerboard",
            NoiseType::Cylinders => "Cylinders",
            NoiseType::OpenSimplex => "OpenSimplex",
            NoiseType::Perlin => "Perlin",
            NoiseType::PerlinSurflet => "Perlin surflet",
            NoiseType::Simplex => "Simplex",
            NoiseType::SuperSimplex => "SuperSimplex",
            NoiseType::Value => "Value",
            NoiseType::Worley => "Worley",
        }
    }
}

impl Editable for Parameters {
    fn name(&self) -> &str {
        "Planet generation"
    }

    fn widget(&mut self, ui: &Ui) {
        ui.slider("Resolution", 1, 10000, &mut self.resolution);
        ui.slider("Radius", 10., 200., &mut self.radius);
        ui.combo("Noise type", &mut self.noise_type, &NOISE_TYPES, |nt| {
            nt.name().into()
        });
        ui.slider("Noise magnitude", 0., 100., &mut self.noise_magnitude);
        ui.slider("Noise scale", 0., 64., &mut self.noise_scale);
        ui.slider("Noise layers", 0, 16, &mut self.noise_layers);
    }
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            resolution: 400,
            radius: 100.,
            noise_type: 3,
            noise_magnitude: 8.,
            noise_scale: 6.,
            noise_layers: 4,
        }
    }
}

pub fn generate_planet(parameters: &Parameters) -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let noise = select_noise(parameters);
    for side in SIDES {
        for i in 0..parameters.resolution {
            for j in 0..parameters.resolution {
                let position_bottom_left = generate_vertex(i, j, &side, &parameters, &noise);
                let position_bottom_right = generate_vertex(i + 1, j, &side, &parameters, &noise);
                let position_top_left = generate_vertex(i, j + 1, &side, &parameters, &noise);
                let position_top_right = generate_vertex(i + 1, j + 1, &side, &parameters, &noise);
                let normal_first = (position_bottom_right - position_bottom_left)
                    .cross(&(position_top_left - position_bottom_left));
                let normal_second = (position_bottom_right - position_top_left)
                    .cross(&(position_top_right - position_top_left));
                let tex = Vector2::zeros();
                vertices.push(Vertex {
                    position: position_bottom_left,
                    normal: normal_first,
                    tex,
                });
                vertices.push(Vertex {
                    position: position_bottom_right,
                    normal: normal_first,
                    tex,
                });
                vertices.push(Vertex {
                    position: position_top_left,
                    normal: normal_first,
                    tex,
                });
                vertices.push(Vertex {
                    position: position_top_left,
                    normal: normal_second,
                    tex,
                });
                vertices.push(Vertex {
                    position: position_bottom_right,
                    normal: normal_second,
                    tex,
                });
                vertices.push(Vertex {
                    position: position_top_right,
                    normal: normal_second,
                    tex,
                });
            }
        }
    }
    for i in 0..vertices.len() as u32 {
        indices.push(i);
    }
    Model {
        vertices,
        indices,
        texture_path: "assets/cube.png",
    }
}

fn select_noise(parameters: &Parameters) -> Box<dyn NoiseFn<f64, 2>> {
    let seed = 907;
    match NOISE_TYPES[parameters.noise_type] {
        NoiseType::Checkerboard => Box::new(Checkerboard::new(1)),
        NoiseType::Cylinders => Box::new(Cylinders::new()),
        NoiseType::OpenSimplex => Box::new(OpenSimplex::new(seed)),
        NoiseType::Perlin => Box::new(Perlin::new(seed)),
        NoiseType::PerlinSurflet => Box::new(PerlinSurflet::new(seed)),
        NoiseType::Simplex => Box::new(Simplex::new(seed)),
        NoiseType::SuperSimplex => Box::new(SuperSimplex::new(seed)),
        NoiseType::Value => Box::new(Value::new(seed)),
        NoiseType::Worley => Box::new(Worley::new(seed)),
    }
}

fn generate_vertex(
    i: usize,
    j: usize,
    side: &Side,
    parameters: &Parameters,
    noise: &dyn NoiseFn<f64, 2>,
) -> Vector3<f32> {
    let direction = (side.base
        + i as f32 * (2. * side.dx) / parameters.resolution as f32
        + j as f32 * (2. * side.dy) / parameters.resolution as f32)
        .normalize();
    let is_on_edge = i == 0 || i == parameters.resolution || j == 0 || j == parameters.resolution;
    let noise_x = parameters.noise_scale * i as f32 / parameters.resolution as f32;
    let noise_y = parameters.noise_scale * j as f32 / parameters.resolution as f32;
    let mut noise_value = 0.;
    for i in 0..parameters.noise_layers {
        let factor = (2.0f64).powi(i as i32);
        noise_value +=
            noise.get([noise_x as f64 * factor, noise_y as f64 * factor]) as f32 / factor as f32;
    }
    let position = direction
        * (parameters.radius
            + if is_on_edge {
                0.
            } else {
                parameters.noise_magnitude * noise_value
            });
    position
}
