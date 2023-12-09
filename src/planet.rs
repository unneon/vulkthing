use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use log::debug;
use nalgebra::Vector3;
use noise::{NoiseFn, Perlin};
use rand::random;

#[derive(Clone, PartialEq)]
pub struct Planet {
    pub resolution: usize,
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

pub fn generate_planet(parameters: &Planet) -> MeshData {
    let mut vertices = Vec::new();
    let noise_seed = random();
    let noise = Perlin::new(noise_seed);
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
    debug!(
        "planet model generated, \x1B[1mvertices\x1B[0m: {}",
        vertices.len()
    );
    MeshData { vertices }
}

fn generate_vertex(
    i: usize,
    j: usize,
    side: &Side,
    planet: &Planet,
    noise: &Perlin,
) -> Vector3<f32> {
    let cube_point = side.base
        + i as f32 * 2. * side.dx / planet.resolution as f32
        + j as f32 * 2. * side.dy / planet.resolution as f32;
    let sphere_point = cube_point.normalize();
    let sample = sphere_point * planet.noise_scale;
    let noise_value = noise.get([sample.x as f64, sample.y as f64, sample.z as f64]);
    sphere_point * (1. + planet.noise_magnitude * noise_value as f32)
}
