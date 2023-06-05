use crate::input::InputState;
use crate::{CAMERA_SENSITIVITY, SPRINT_SPEED, WALK_SPEED};
use nalgebra::{Matrix4, Point3, UnitQuaternion, Vector3};

pub mod first_person;

#[allow(dead_code)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
}

#[allow(dead_code)]
impl Camera {
    pub fn apply_input(&mut self, input: &InputState, delta_time: f32) {
        let front = self.front_direction();
        let up = self.up_direction();
        let right = front.cross(&up);
        let movement_speed = if input.movement_sprint() {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };
        self.position +=
            normalize_or_zero(right * input.movement_horizontal() + front * input.movement_depth())
                * movement_speed
                * delta_time;
        self.rotation *= UnitQuaternion::from_euler_angles(
            input.camera_roll() * delta_time,
            input.camera_pitch() * CAMERA_SENSITIVITY,
            -input.camera_yaw() * CAMERA_SENSITIVITY,
        );
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let eye = Point3::from(self.position);
        let target = Point3::from(self.position + self.front_direction());
        let up = self.up_direction();
        Matrix4::look_at_rh(&eye, &target, &up)
    }

    fn front_direction(&self) -> Vector3<f32> {
        self.rotation.to_rotation_matrix() * Vector3::new(1., 0., 0.)
    }

    fn up_direction(&self) -> Vector3<f32> {
        self.rotation.to_rotation_matrix() * Vector3::new(0., 0., 1.)
    }
}

fn normalize_or_zero(vec: Vector3<f32>) -> Vector3<f32> {
    if let Some(normalized) = vec.try_normalize(1.0e-6) {
        normalized
    } else {
        Vector3::zeros()
    }
}
