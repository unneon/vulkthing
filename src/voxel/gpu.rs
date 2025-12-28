pub mod meshlets;

use crate::gpu::SvoNode;
use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::material::Material;
use crate::voxel::sparse_octree::SparseOctree;
use nalgebra::Vector3;

pub trait VoxelGpuMemory: Send + 'static {
    fn prepare_func(&self) -> fn(LocalMesh, &SparseOctree, Vector3<i64>) -> Box<dyn std::any::Any>;

    fn upload(&mut self, prepared: Box<dyn std::any::Any>);

    fn clear(&mut self);

    fn cleanup(&mut self);
}

pub const EMPTY_ROOT: SvoNode = SvoNode {
    children: [svo_uniform(Material::Air); 8],
    parent: 0,
};

pub const fn svo_uniform(material: Material) -> u32 {
    (1 << 31) | (material as u8 as u32)
}

pub const fn svo_mixed(pointer: u32) -> u32 {
    assert!(pointer < (1 << 31));
    pointer
}
