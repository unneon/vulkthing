use nalgebra::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GrassBlade {
    pub position: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub front: Vector3<f32>,
    pub width: f32,
    pub height: f32,
    pub ground_normal: Vector3<f32>,
}
