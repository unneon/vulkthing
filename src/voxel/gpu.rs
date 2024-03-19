pub mod meshlets;

use crate::voxel::local_mesh::LocalMesh;
use nalgebra::Vector3;

pub trait VoxelGpuMemory: Send + 'static {
    fn prepare_func(&self) -> fn(LocalMesh, Vector3<i64>) -> Box<dyn std::any::Any>;

    fn upload(&mut self, prepared: Box<dyn std::any::Any>);

    fn clear(&mut self);

    fn cleanup(&mut self);
}
