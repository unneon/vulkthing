use nalgebra::Vector3;

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
