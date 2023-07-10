use crate::camera::space::SpaceCamera;
use crate::world::World;
use nalgebra::{UnitQuaternion, Vector3};
use std::f32::consts::PI;

impl World {
    pub fn update_benchmark(&mut self, frame_index: usize) {
        let radius_time = 0.002 * frame_index as f32;
        let angle_time = 0.0024 * frame_index as f32;
        let camera_time = 0.001 * frame_index as f32;
        let radius = 1050. + 1000. * (1. - radius_time).max(0.);
        let angle = angle_time;
        let camera = angle - 0.8 * PI / 2. * (1. - (1. - camera_time).max(0.));
        let x = -radius * angle.cos();
        let y = 0.;
        let z = radius * angle.sin();
        self.camera = Box::new(SpaceCamera {
            position: Vector3::new(x, y, z),
            rotation: UnitQuaternion::from_euler_angles(0., camera, 0.),
        });
    }
}
