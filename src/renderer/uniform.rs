use nalgebra::{Matrix4, Vector3};

#[repr(C)]
pub struct ModelViewProjection {
    pub model: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

#[repr(C)]
pub struct Material {
    pub emit: Vector3<f32>,
}

#[repr(C)]
pub struct Light {
    pub color: Vector3<f32>,
    pub ambient_strength: f32,
    pub position: Vector3<f32>,
}
