pub mod meshlets;

use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::material::Material;
use nalgebra::Vector3;

pub trait VoxelGpuMemory: Send + 'static {
    fn prepare_func(&self) -> fn(LocalMesh, Vector3<i64>) -> Box<dyn std::any::Any>;

    fn upload(&mut self, prepared: Box<dyn std::any::Any>);

    fn clear(&mut self);

    fn cleanup(&mut self);
}

#[derive(Clone, Copy)]
pub struct SparseVoxelOctree {
    #[allow(dead_code)]
    packed: u32,
}

impl SparseVoxelOctree {
    pub fn new_uniform(material: Material) -> SparseVoxelOctree {
        SparseVoxelOctree {
            packed: (1 << 31) | (material as u8 as u32),
        }
    }

    pub fn new_mixed(pointer: u32) -> SparseVoxelOctree {
        assert!(pointer < (1 << 31));
        SparseVoxelOctree { packed: pointer }
    }
}
