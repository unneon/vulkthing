use nalgebra::{UnitQuaternion, Vector3};
use rand::distributions::Distribution;
use rand::Rng;
use std::f32::consts::PI;

pub struct RandomDirection;

pub struct RandomRotation;

impl Distribution<Vector3<f32>> for RandomDirection {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vector3<f32> {
        // https://math.stackexchange.com/a/44691
        let theta = rng.gen_range((0.)..2. * PI);
        let z: f32 = rng.gen_range((-1.)..1.);
        Vector3::new(
            (1. - z * z).sqrt() * theta.cos(),
            (1. - z * z).sqrt() * theta.sin(),
            z,
        )
    }
}

impl Distribution<UnitQuaternion<f32>> for RandomRotation {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> UnitQuaternion<f32> {
        // TODO: This probably isn't a uniform distribution.
        let roll = rng.gen_range((0.)..2. * PI);
        let pitch = rng.gen_range((0.)..2. * PI);
        let yaw = rng.gen_range((0.)..2. * PI);
        UnitQuaternion::from_euler_angles(roll, pitch, yaw)
    }
}
