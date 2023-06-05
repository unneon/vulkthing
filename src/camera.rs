use crate::input::InputState;
use nalgebra::{Matrix4, Vector3};

pub mod first_person;
pub mod space;

pub trait Camera {
    fn apply_input(&mut self, input: &InputState, delta_time: f32);

    fn position(&self) -> Vector3<f32>;

    fn set_position(&mut self, position: Vector3<f32>);

    fn view_matrix(&self) -> Matrix4<f32>;

    fn walk_direction(&self) -> Vector3<f32>;
}

fn normalize_or_zero(vec: Vector3<f32>) -> Vector3<f32> {
    if let Some(normalized) = vec.try_normalize(1.0e-6) {
        normalized
    } else {
        Vector3::zeros()
    }
}
