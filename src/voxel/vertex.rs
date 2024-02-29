use nalgebra::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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
