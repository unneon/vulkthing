use crate::voxel::material::Material;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::DIRECTIONS;
use nalgebra::Vector3;
use std::sync::Arc;

pub struct Neighbourhood<'a> {
    svos: [&'a SparseOctree; 27],
    chunk_size: i64,
}

impl<'a> Neighbourhood<'a> {
    pub fn new(svos: &'a [Arc<SparseOctree>], chunk_size: i64) -> Neighbourhood<'a> {
        let svos = std::array::from_fn(|i| &*svos[i]);
        Neighbourhood { svos, chunk_size }
    }

    pub fn at(&self, mut position: Vector3<i64>) -> Material {
        let mut chunk = Vector3::new(0, 0, 0);
        if position.x < 0 {
            chunk.x = -1;
            position.x += self.chunk_size;
        } else if position.x >= self.chunk_size {
            chunk.x = 1;
            position.x -= self.chunk_size;
        }
        if position.y < 0 {
            chunk.y = -1;
            position.y += self.chunk_size;
        } else if position.y >= self.chunk_size {
            chunk.y = 1;
            position.y -= self.chunk_size;
        }
        if position.z < 0 {
            chunk.z = -1;
            position.z += self.chunk_size;
        } else if position.z >= self.chunk_size {
            chunk.z = 1;
            position.z -= self.chunk_size;
        }
        self.chunk_at(chunk).at(position, self.chunk_size)
    }

    pub fn chunk(&self) -> &'a SparseOctree {
        self.chunk_at(Vector3::new(0, 0, 0))
    }

    pub fn chunk_at(&self, chunk: Vector3<i64>) -> &'a SparseOctree {
        self.svos[(9 * (chunk.z + 1) + 3 * (chunk.y + 1) + (chunk.x + 1)) as usize]
    }

    pub fn neighbour_chunks_manhattan(&self) -> [&'a SparseOctree; 6] {
        DIRECTIONS.map(|direction| self.chunk_at(direction))
    }
}
