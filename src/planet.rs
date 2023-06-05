use crate::interface::EnumInterface;
use crate::model::Model;
use crate::renderer::vertex::Vertex;
use nalgebra::Vector3;
use noise::{
    NoiseFn, OpenSimplex, Perlin, PerlinSurflet, RidgedMulti, Simplex, SuperSimplex, Value,
};
use rand::random;
use std::borrow::Cow;

#[derive(Clone, PartialEq)]
pub struct Planet {
    pub resolution: usize,
    pub radius: f32,
    pub noise_type: NoiseType,
    pub noise_magnitude: f32,
    pub noise_scale: f32,
    pub noise_layers: usize,
    pub chunk_count: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NoiseType {
    OpenSimplex,
    Perlin,
    PerlinSurflet,
    Ridge,
    Simplex,
    SuperSimplex,
    Value,
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

impl EnumInterface for NoiseType {
    const VALUES: &'static [Self] = &[
        NoiseType::OpenSimplex,
        NoiseType::Perlin,
        NoiseType::PerlinSurflet,
        NoiseType::Ridge,
        NoiseType::Simplex,
        NoiseType::SuperSimplex,
        NoiseType::Value,
    ];

    fn label(&self) -> Cow<str> {
        Cow::Borrowed(match self {
            NoiseType::OpenSimplex => "OpenSimplex",
            NoiseType::Perlin => "Perlin",
            NoiseType::PerlinSurflet => "Perlin surflet",
            NoiseType::Ridge => "Ridge",
            NoiseType::Simplex => "Simplex",
            NoiseType::SuperSimplex => "SuperSimplex",
            NoiseType::Value => "Value",
        })
    }
}

pub fn generate_planet(parameters: &Planet) -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let noise = select_noise(parameters);
    for side in SIDES {
        for i in 0..parameters.resolution {
            for j in 0..parameters.resolution {
                let position_bottom_left = generate_vertex(i, j, &side, parameters, &noise);
                let position_bottom_right = generate_vertex(i + 1, j, &side, parameters, &noise);
                let position_top_left = generate_vertex(i, j + 1, &side, parameters, &noise);
                let position_top_right = generate_vertex(i + 1, j + 1, &side, parameters, &noise);
                let normal_first = (position_bottom_right - position_bottom_left)
                    .cross(&(position_top_left - position_bottom_left))
                    .normalize();
                let normal_second = (position_bottom_right - position_top_left)
                    .cross(&(position_top_right - position_top_left))
                    .normalize();
                vertices.push(Vertex {
                    position: position_bottom_left,
                    normal: normal_first,
                });
                vertices.push(Vertex {
                    position: position_bottom_right,
                    normal: normal_first,
                });
                vertices.push(Vertex {
                    position: position_top_left,
                    normal: normal_first,
                });
                vertices.push(Vertex {
                    position: position_top_left,
                    normal: normal_second,
                });
                vertices.push(Vertex {
                    position: position_bottom_right,
                    normal: normal_second,
                });
                vertices.push(Vertex {
                    position: position_top_right,
                    normal: normal_second,
                });
            }
        }
    }
    for i in 0..vertices.len() as u32 {
        indices.push(i);
    }
    Model { vertices, indices }
}

fn select_noise(parameters: &Planet) -> Box<dyn NoiseFn<f64, 3>> {
    let seed = random();
    match parameters.noise_type {
        NoiseType::OpenSimplex => Box::new(OpenSimplex::new(seed)),
        NoiseType::Perlin => Box::new(Perlin::new(seed)),
        NoiseType::PerlinSurflet => Box::new(PerlinSurflet::new(seed)),
        NoiseType::Ridge => Box::new(RidgedMulti::<Perlin>::new(seed)),
        NoiseType::Simplex => Box::new(Simplex::new(seed)),
        NoiseType::SuperSimplex => Box::new(SuperSimplex::new(seed)),
        NoiseType::Value => Box::new(Value::new(seed)),
    }
}

fn generate_vertex(
    i: usize,
    j: usize,
    side: &Side,
    parameters: &Planet,
    noise: &dyn NoiseFn<f64, 3>,
) -> Vector3<f32> {
    let direction = (side.base
        + i as f32 * (2. * side.dx) / parameters.resolution as f32
        + j as f32 * (2. * side.dy) / parameters.resolution as f32)
        .normalize();
    let mut noise_value = 0.;
    for i in 0..parameters.noise_layers {
        let factor = (2.0f64).powi(i as i32);
        noise_value += noise.get([
            direction.x as f64 * factor,
            direction.y as f64 * factor,
            direction.z as f64 * factor,
        ]) as f32
            / factor as f32;
    }
    direction * (parameters.radius + parameters.noise_magnitude * noise_value)
}
