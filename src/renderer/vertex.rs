use nalgebra::{Matrix4, Vector3};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelVertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub material: u16,
    pub _pad0: [u8; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GrassBlade {
    pub position: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub front: Vector3<f32>,
    pub height_noise: f32,
    pub ground_normal: Vector3<f32>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Star {
    pub model: Matrix4<f32>,
    pub emit: Vector3<f32>,
}
