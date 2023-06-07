use crate::model::Model;
use crate::planet::Planet;
use crate::renderer::vertex::GrassBlade;
use nalgebra::{Rotation3, Unit, Vector3};
use noise::{NoiseFn, Perlin};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;

#[derive(Clone)]
pub struct Grass {
    pub blades_per_triangle: usize,
    pub height_average: f32,
    pub height_max_variance: f32,
    pub height_noise_frequency: f32,
    pub width: f32,
    pub sway_frequency: f32,
    pub sway_amplitude: f32,
    pub chunk_count: usize,
    pub chunk_load_distance: f32,
    pub chunk_unload_distance: f32,
}

const GOLDEN_RATIO: f32 = 1.618034;

pub fn generate_grass_blades(
    grass: &Grass,
    planet_model: &Model,
    chunk: &[usize],
) -> Vec<GrassBlade> {
    let mut grass_blades = Vec::new();
    let mut rng = SmallRng::from_seed([23; 32]);
    let height_noise_generator = Perlin::new(907);
    for triangle_index in chunk {
        let triangle = &planet_model.vertices[3 * triangle_index..3 * triangle_index + 3];
        let d1 = triangle[1].position - triangle[0].position;
        let d2 = triangle[2].position - triangle[0].position;
        for _ in 0..grass.blades_per_triangle {
            let mut t1: f32 = rng.gen();
            let mut t2: f32 = rng.gen();
            if t1 + t2 > 1. {
                t1 = 1. - t1;
                t2 = 1. - t2;
            }
            let position = triangle[0].position + t1 * d1 + t2 * d2;
            let up = position.normalize();
            let angle = rng.gen_range((0.)..(2. * PI));
            // https://math.stackexchange.com/a/4112622
            let right = (Rotation3::from_axis_angle(&Unit::new_normalize(up), angle)
                * Vector3::new(
                    up.z.copysign(up.x),
                    up.z.copysign(up.y),
                    -(up.x.abs() + up.y.abs()).copysign(up.z),
                ))
            .normalize();
            let front = up.cross(&right).normalize();
            let height_noise_arg = position * grass.height_noise_frequency;
            let height_noise = height_noise_generator.get([
                height_noise_arg.x as f64,
                height_noise_arg.y as f64,
                height_noise_arg.z as f64,
            ]) as f32;
            grass_blades.push(GrassBlade {
                position,
                up,
                right,
                front,
                height_noise,
                ground_normal: triangle[0].normal,
            });
        }
    }
    grass_blades
}

pub fn build_triangle_chunks(
    grass: &Grass,
    planet: &Planet,
    planet_model: &Model,
) -> Vec<Vec<usize>> {
    assert_eq!(grass.chunk_count % 2, 1);
    let fib = compute_fibonacci_sphere(grass.chunk_count as i64 / 2);
    let mut chunks = vec![Vec::new(); grass.chunk_count];
    for (i, triangle) in planet_model.vertices.array_chunks::<3>().enumerate() {
        // I love how Rust doesn't implement Ord on floats.
        let mut best_distance = f32::INFINITY;
        let mut best_chunk_id = usize::MAX;
        for (chunk_id, chunk_center) in fib.iter().enumerate() {
            let distance = (chunk_center.scale(planet.radius) - triangle[0].position).norm();
            if distance < best_distance {
                best_distance = distance;
                best_chunk_id = chunk_id;
            }
        }
        chunks[best_chunk_id].push(i);
    }
    chunks
}

/// Generates 2n+1 reasonably uniformly distributed points on the unit sphere.
fn compute_fibonacci_sphere(n: i64) -> Vec<Vector3<f32>> {
    // https://arxiv.org/pdf/0912.4540.pdf
    let mut points = Vec::new();
    for i in -n..=n {
        let latitude = ((2 * i) as f32 / (2 * n + 1) as f32).asin();
        let longitude = 2. * PI * (i as f32) * GOLDEN_RATIO;
        let position = Vector3::new(
            longitude.cos() * latitude.cos(),
            longitude.sin() * latitude.cos(),
            latitude.sin(),
        );
        points.push(position);
    }
    points
}
