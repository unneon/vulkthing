use crate::input::InputState;
use nalgebra_glm as glm;
use nalgebra_glm::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
}

impl Camera {
    pub fn apply_input(&mut self, input: &InputState, delta_time: f32) {
        let front_direction = self.view_direction();
        let right_direction = glm::vec3(front_direction.y, -front_direction.x, 0.);
        self.position += (right_direction * input.movement_horizontal()
            + front_direction * input.movement_depth())
            * delta_time;
        self.yaw += input.camera_yaw() * delta_time;
    }

    pub fn view_matrix(&self) -> Mat4 {
        let view_center = self.position + self.view_direction();
        glm::look_at(&self.position, &view_center, &glm::vec3(0., 0., 1.))
    }

    fn view_direction(&self) -> Vec3 {
        glm::vec3(self.yaw.sin(), self.yaw.cos(), 0.)
    }
}
