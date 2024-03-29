pub mod meshlets;

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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct SvoNode {
    children: [SvoChild; 8],
    parent: u32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub struct SvoChild {
    packed: u32,
}

impl SvoNode {
    pub const EMPTY_ROOT: SvoNode = SvoNode {
        children: [SvoChild::new_uniform(Material::Air); 8],
        parent: 0,
    };

    pub fn new(parent: u32, children: [SvoChild; 8]) -> SvoNode {
        SvoNode { children, parent }
    }
}

impl SvoChild {
    pub const fn new_uniform(material: Material) -> SvoChild {
        SvoChild {
            packed: (1 << 31) | (material as u8 as u32),
        }
    }

    pub const fn new_mixed(pointer: u32) -> SvoChild {
        assert!(pointer < (1 << 31));
        SvoChild { packed: pointer }
    }
}
