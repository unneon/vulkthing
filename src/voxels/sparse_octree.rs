use crate::voxels::VoxelKind;
use nalgebra::Vector3;

#[derive(Clone, Debug)]
pub enum SparseOctree {
    Uniform { kind: VoxelKind },
    Mixed { children: [Box<SparseOctree>; 8] },
}

impl SparseOctree {
    pub fn at(&self, point: Vector3<i64>, local_size: i64) -> VoxelKind {
        match self {
            SparseOctree::Uniform { kind } => *kind,
            SparseOctree::Mixed { children } => {
                let child_size = local_size / 2;
                let mut index = 0;
                if point.z >= child_size {
                    index += 4;
                }
                if point.y >= child_size {
                    index += 2;
                }
                if point.x >= child_size {
                    index += 1;
                }
                let child_coordinates = Vector3::new(
                    point.x % child_size,
                    point.y % child_size,
                    point.z % child_size,
                );
                children[index].at(child_coordinates, child_size)
            }
        }
    }
}
