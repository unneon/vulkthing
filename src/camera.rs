use crate::input::InputState;
use crate::{CAMERA_SENSITIVITY, MOVEMENT_SPEED};
use nalgebra_glm as glm;
use nalgebra_glm::{vec3, Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
}

impl Camera {
    pub fn apply_input(&mut self, input: &InputState, delta_time: f32) {
        let front = self.walk_direction();
        let up = vec3(0., 0., 1.);
        let right = glm::cross(&front, &up);
        self.position +=
            normalize_or_zero(right * input.movement_horizontal() + front * input.movement_depth())
                * MOVEMENT_SPEED
                * delta_time;
        self.yaw += input.camera_yaw() * CAMERA_SENSITIVITY;
    }

    pub fn view_matrix(&self) -> Mat4 {
        let view_center = self.position + self.walk_direction();
        glm::look_at(&self.position, &view_center, &vec3(0., 0., 1.))
    }

    fn walk_direction(&self) -> Vec3 {
        vec3(self.yaw.cos(), -self.yaw.sin(), 0.)
    }
}

fn normalize_or_zero(vec: Vec3) -> Vec3 {
    if let Some(normalized) = vec.try_normalize(1.0e-6) {
        normalized
    } else {
        glm::zero()
    }
}
