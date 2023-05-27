use crate::interface::Editable;
use crate::model::Model;
use crate::renderer::vertex::Vertex;
use imgui::Ui;
use nalgebra::Vector3;
use noise::{
    NoiseFn, OpenSimplex, Perlin, PerlinSurflet, RidgedMulti, Simplex, SuperSimplex, Value,
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

const NOISE_TYPES: &[NoiseType] = &[
    NoiseType::OpenSimplex,
    NoiseType::Perlin,
    NoiseType::PerlinSurflet,
    NoiseType::Ridge,
    NoiseType::Simplex,
    NoiseType::SuperSimplex,
    NoiseType::Value,
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
            NoiseType::OpenSimplex => "OpenSimplex",
            NoiseType::Perlin => "Perlin",
            NoiseType::PerlinSurflet => "Perlin surflet",
            NoiseType::Ridge => "Ridge",
            NoiseType::Simplex => "Simplex",
            NoiseType::SuperSimplex => "SuperSimplex",
            NoiseType::Value => "Value",
        }
    }
}

impl Editable for Parameters {
    fn name(&self) -> &str {
        "Planet generation"
    }

    fn widget(&mut self, ui: &Ui) -> bool {
        let mut changed = false;
        changed |= ui.slider("Resolution", 1, 800, &mut self.resolution);
        changed |= ui.slider("Radius", 10., 200., &mut self.radius);
        changed |= ui.combo("Noise type", &mut self.noise_type, NOISE_TYPES, |nt| {
            nt.name().into()
        });
        changed |= ui.slider("Noise magnitude", 0., 100., &mut self.noise_magnitude);
        changed |= ui.slider("Noise scale", 0., 64., &mut self.noise_scale);
        changed |= ui.slider("Noise layers", 0, 16, &mut self.noise_layers);
        changed
    }
}

impl Default for Parameters {
    fn default() -> Parameters {
        Parameters {
            resolution: 400,
            radius: 100.,
            noise_type: 3,
            noise_magnitude: 20.,
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

fn select_noise(parameters: &Parameters) -> Box<dyn NoiseFn<f64, 3>> {
    let seed = 907;
    match NOISE_TYPES[parameters.noise_type] {
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
    parameters: &Parameters,
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
