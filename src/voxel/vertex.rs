use nalgebra::Vector3;
use std::hash::{Hash, Hasher};
use std::intrinsics::transmute;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VoxelVertex {
    pub position: Vector3<f32>,
    _pad0: f32,
}

impl VoxelVertex {
    pub fn new(position: Vector3<f32>) -> VoxelVertex {
        VoxelVertex {
            position,
            _pad0: 0.,
        }
    }
}

impl Eq for VoxelVertex {}

impl Hash for VoxelVertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // TODO: This is awful. How do you properly compare floats in Rust?
        type Raw = [u8; std::mem::size_of::<VoxelVertex>()];
        <Raw as Hash>::hash(unsafe { transmute::<&VoxelVertex, &Raw>(self) }, state)
    }
}
