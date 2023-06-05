use crate::model::Model;
use crate::planet::Planet;
use crate::renderer::vertex::GrassBlade;
use log::debug;
use nalgebra::{Rotation3, Unit, Vector3};
use noise::{NoiseFn, Perlin};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;

pub struct Grass {
    pub render_distance: f32,
    pub blades_per_triangle: usize,
    pub height_average: f32,
    pub height_max_variance: f32,
    pub height_noise_frequency: f32,
    pub width: f32,
}

pub fn generate_grass_blades(
    grass: &Grass,
    planet: &Planet,
    planet_model: &Model,
) -> Vec<GrassBlade> {
    let mut grass_blades = Vec::new();
    let mut rng = SmallRng::from_seed([23; 32]);
    let height_noise = Perlin::new(907);
    for triangle in planet_model.vertices.array_chunks::<3>() {
        if (triangle[0].position - Vector3::new(0., 0., planet.radius)).norm()
            > grass.render_distance
        {
            continue;
        }
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
            let height = grass.height_average
                + grass.height_max_variance
                    * height_noise.get([
                        height_noise_arg.x as f64,
                        height_noise_arg.y as f64,
                        height_noise_arg.z as f64,
                    ]) as f32;
            grass_blades.push(GrassBlade {
                position,
                up,
                right,
                front,
                width: grass.width,
                height,
                ground_normal: triangle[0].normal,
            });
        }
    }
    debug!(
        "grass blades generated, \x1B[1mcount\x1B[0m: {}",
        grass_blades.len()
    );
    grass_blades
}
