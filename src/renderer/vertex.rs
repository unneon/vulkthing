use nalgebra::Vector3;
use std::hash::{Hash, Hasher};
use std::mem::transmute;

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

impl Eq for VoxelVertex {}

impl Hash for VoxelVertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        type Raw = [u8; std::mem::size_of::<VoxelVertex>()];
        <Raw as Hash>::hash(unsafe { transmute::<&VoxelVertex, &Raw>(self) }, state)
    }
}
