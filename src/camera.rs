use crate::input::InputState;
use crate::{CAMERA_SENSITIVITY, SPRINT_SPEED, WALK_SPEED};
use nalgebra::{Matrix4, Point3, Vector3};
use std::f32::consts::FRAC_PI_2;

pub struct Camera {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub yaw: f32,
    pub pitch: f32,
}

const YAW_LIMIT: f32 = FRAC_PI_2 - 0.00001;

impl Camera {
    pub fn apply_input(&mut self, input: &InputState, delta_time: f32) {
        let front = self.walk_direction();
        let up = Vector3::new(0., 0., 1.);
        let right = front.cross(&up);
        let movement_speed = if input.movement_sprint() {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        if input.movement_jumps() > 0 && self.position.z == 0. {
            self.velocity.z = 4.;
        }
        self.velocity.z -= 9.807 * delta_time;
        self.position += self.velocity * delta_time;
        if self.position.z < 0. {
            self.position.z = 0.;
            self.velocity.z = 0.;
        }
        self.position +=
            normalize_or_zero(right * input.movement_horizontal() + front * input.movement_depth())
                * movement_speed
                * delta_time;
        self.yaw += input.camera_yaw() * CAMERA_SENSITIVITY;
        self.pitch =
            (self.pitch - input.camera_pitch() * CAMERA_SENSITIVITY).clamp(-YAW_LIMIT, YAW_LIMIT);
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = Point3::from(self.position);
        let target = Point3::from(self.position + self.view_direction());
        let up = Vector3::new(0., 0., 1.);
        Matrix4::look_at_rh(&eye, &target, &up)
    }

    fn walk_direction(&self) -> Vector3<f32> {
        Vector3::new(self.yaw.cos(), -self.yaw.sin(), 0.)
    }

    fn view_direction(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
    }
}

fn normalize_or_zero(vec: Vector3<f32>) -> Vector3<f32> {
    if let Some(normalized) = vec.try_normalize(1.0e-6) {
        normalized
    } else {
        Vector3::zeros()
    }
}
