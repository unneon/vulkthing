use crate::input::InputState;
use crate::{CAMERA_SENSITIVITY, MOVEMENT_SPEED};
use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Mat4, Vec3};
use std::f32::consts::FRAC_PI_2;

pub struct Camera {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

const YAW_LIMIT: f32 = FRAC_PI_2 - 0.00001;

impl Camera {
    pub fn apply_input(&mut self, input: &InputState, delta_time: f32) {
        let front = self.walk_direction();
        let up = vec3(0., 0., 1.);
        let right = glm::cross(&front, &up);
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
                * MOVEMENT_SPEED
                * delta_time;
        self.yaw += input.camera_yaw() * CAMERA_SENSITIVITY;
        self.pitch =
            (self.pitch - input.camera_pitch() * CAMERA_SENSITIVITY).clamp(-YAW_LIMIT, YAW_LIMIT);
    }

    pub fn view_matrix(&self) -> Mat4 {
        let view_center = self.position + self.view_direction();
        glm::look_at(&self.position, &view_center, &vec3(0., 0., 1.))
    }

    fn walk_direction(&self) -> Vec3 {
        vec3(self.yaw.cos(), -self.yaw.sin(), 0.)
    }

    fn view_direction(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            -self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
    }
}

fn normalize_or_zero(vec: Vec3) -> Vec3 {
    if let Some(normalized) = vec.try_normalize(1.0e-6) {
        normalized
    } else {
        glm::zero()
    }
}
