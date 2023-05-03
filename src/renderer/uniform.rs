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

#[repr(C)]
pub struct Filters {
    pub color_filter: Vector3<f32>,
    pub exposure: f32,
    pub temperature: f32,
    pub tint: f32,
    pub contrast: f32,
    pub brightness: f32,
    pub saturation: f32,
    pub gamma: f32,
}

impl Default for Filters {
    fn default() -> Self {
        Filters {
            exposure: 1.,
            temperature: 0.,
            tint: 0.,
            contrast: 1.,
            brightness: 0.,
            color_filter: Vector3::new(1., 1., 1.),
            saturation: 1.,
            gamma: 1.,
        }
    }
}
