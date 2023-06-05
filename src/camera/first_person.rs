use crate::camera::normalize_or_zero;
use crate::input::InputState;
use crate::CAMERA_SENSITIVITY;
use nalgebra::{Matrix4, Point3, Vector3};
use std::f32::consts::{FRAC_PI_2, PI};

pub struct FirstPersonCamera {
    pub position: Vector3<f32>,
    pub walk_direction: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
}

impl FirstPersonCamera {
    pub fn apply_input(&mut self, input: &InputState) {
        let front = self.walk_direction();
        let up = self.up_direction();
        let right = front.cross(&up);
        self.walk_direction =
            normalize_or_zero(right * input.movement_horizontal() + front * input.movement_depth());
        self.pitch = (self.pitch - input.camera_pitch() * CAMERA_SENSITIVITY)
            .clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);
        self.yaw = (self.yaw - input.camera_yaw() * CAMERA_SENSITIVITY).rem_euclid(2. * PI);
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = Point3::from(self.position);
        let target = Point3::from(self.position + self.front_direction());
        let up = self.up_direction();
        Matrix4::look_at_rh(&eye, &target, &up)
    }

    fn front_direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
    }

    fn walk_direction(&self) -> Vector3<f32> {
        Vector3::new(self.yaw.cos(), self.yaw.sin(), 0.)
    }

    fn up_direction(&self) -> Vector3<f32> {
        Vector3::new(0., 0., 1.)
    }
}
