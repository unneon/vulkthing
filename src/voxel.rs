use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use nalgebra::{DMatrix, Vector2, Vector3};
use noise::{NoiseFn, Perlin};

pub struct Voxels {
    pub chunk_size: usize,
    noise: Perlin,
}

#[derive(Clone, Debug)]
pub enum SparseVoxelOctree {
    Uniform {
        kind: VoxelKind,
    },
    Mixed {
        children: [Box<SparseVoxelOctree>; 8],
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum VoxelKind {
    Air = 0,
    Stone = 1,
}

impl Voxels {
    pub fn new(chunk_size: usize, seed: u32) -> Voxels {
        Voxels {
            chunk_size,
            noise: Perlin::new(seed),
        }
    }

    pub fn generate_chunk_heightmap(&self, chunk: Vector3<i64>) -> DMatrix<i64> {
        let chunk_coordinates = chunk.xy() * self.chunk_size as i64;
        let mut heightmap = DMatrix::from_element(self.chunk_size, self.chunk_size, 0);
        for x in 0..self.chunk_size {
            for y in 0..self.chunk_size {
                let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
                let noise_position = column_coordinates.cast::<f64>() / 128.;
                let noise_arguments: [f64; 2] = noise_position.into();
                let raw_noise = self.noise.get(noise_arguments);
                let scaled_noise = raw_noise * 32.;
                heightmap[(x, y)] = scaled_noise.round() as i64;
                // heightmap[(x, y)] = 1;
            }
        }
        heightmap
    }

    pub fn svo_from_heightmap(
        &self,
        chunk: Vector3<i64>,
        heightmap: &DMatrix<i64>,
    ) -> SparseVoxelOctree {
        assert_eq!(heightmap.nrows(), self.chunk_size);
        assert_eq!(heightmap.ncols(), self.chunk_size);
        svo_from_heightmap_impl(
            0,
            0,
            chunk.z * self.chunk_size as i64,
            self.chunk_size,
            heightmap,
        )
    }

    pub fn triangles_from_voxels(&self, voxels: &SparseVoxelOctree) -> MeshData {
        let mut vertices = Vec::new();
        for x in 0..self.chunk_size as i64 {
            for y in 0..self.chunk_size as i64 {
                for z in 0..self.chunk_size as i64 {
                    if let Some(side) = self.side((x, y, z), (x, y, z + 1), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                    if let Some(side) = self.side((x, y, z), (x, y, z - 1), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                    if let Some(side) = self.side((x, y, z), (x + 1, y, z), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                    if let Some(side) = self.side((x, y, z), (x - 1, y, z), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                    if let Some(side) = self.side((x, y, z), (x, y + 1, z), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                    if let Some(side) = self.side((x, y, z), (x, y - 1, z), voxels) {
                        vertices.extend_from_slice(&side);
                    }
                }
            }
        }
        MeshData { vertices }
    }

    fn side(
        &self,
        from: (i64, i64, i64),
        to: (i64, i64, i64),
        voxels: &SparseVoxelOctree,
    ) -> Option<[Vertex; 3]> {
        if !voxels.at(
            Vector3::new(from.0 as usize, from.1 as usize, from.2 as usize),
            self.chunk_size,
        ) {
            return None;
        }
        let to_out_of_bounds = (to.0 < 0 || to.0 >= self.chunk_size as i64)
            || (to.1 < 0 || to.1 >= self.chunk_size as i64)
            || (to.2 < 0 || to.2 >= self.chunk_size as i64);
        if !to_out_of_bounds
            && voxels.at(
                Vector3::new(to.0 as usize, to.1 as usize, to.2 as usize),
                self.chunk_size,
            )
        {
            return None;
        }
        let normal = Vector3::new(
            (to.0 - from.0) as f32,
            (to.1 - from.1) as f32,
            (to.2 - from.2) as f32,
        );
        let rot1 = Vector3::new(normal.z.abs(), normal.x.abs(), normal.y.abs());
        let rot2 = Vector3::new(normal.y.abs(), normal.z.abs(), normal.x.abs());
        let base = Vector3::new(from.0 as f32, from.1 as f32, from.2 as f32);
        let base = if normal.x + normal.y + normal.z > 0. {
            base + normal
        } else {
            base
        };
        let (rot1, rot2) = if normal == rot1.cross(&rot2) {
            (rot1, rot2)
        } else {
            (rot2, rot1)
        };
        let v1 = Vertex {
            position: base,
            normal,
        };
        let v2 = Vertex {
            position: base + rot1,
            normal,
        };
        let v3 = Vertex {
            position: base + rot2,
            normal,
        };
        Some([v1, v2, v3])
    }
}

impl SparseVoxelOctree {
    pub fn at(&self, local_coordinates: Vector3<usize>, local_size: usize) -> bool {
        match self {
            SparseVoxelOctree::Uniform { kind } => *kind == VoxelKind::Stone,
            SparseVoxelOctree::Mixed { children } => {
                let child_size = local_size / 2;
                let mut index = 0;
                if local_coordinates.z >= child_size {
                    index += 4;
                }
                if local_coordinates.y >= child_size {
                    index += 2;
                }
                if local_coordinates.x >= child_size {
                    index += 1;
                }
                let child_coordinates = Vector3::new(
                    local_coordinates.x % child_size,
                    local_coordinates.y % child_size,
                    local_coordinates.z % child_size,
                );
                children[index].at(child_coordinates, child_size)
            }
        }
    }
}

fn svo_from_heightmap_impl(
    x: usize,
    y: usize,
    z: i64,
    n: usize,
    heightmap: &DMatrix<i64>,
) -> SparseVoxelOctree {
    'check_all_same: {
        let is_stone = heightmap[(x, y)] > z;
        for ly in y..y + n {
            for lx in x..x + n {
                let height = heightmap[(lx, ly)];
                if height > z && height < z + n as i64 {
                    break 'check_all_same;
                }
                if height <= z && is_stone {
                    break 'check_all_same;
                }
                if height >= z + n as i64 && !is_stone {
                    break 'check_all_same;
                }
            }
        }
        return SparseVoxelOctree::Uniform {
            kind: if is_stone {
                VoxelKind::Stone
            } else {
                VoxelKind::Air
            },
        };
    }
    let mut children = Vec::new();
    for dz in 0..2 {
        for dy in 0..2 {
            for dx in 0..2 {
                children.push(svo_from_heightmap_impl(
                    x + dx * n / 2,
                    y + dy * n / 2,
                    z + dz * n as i64 / 2,
                    n / 2,
                    heightmap,
                ));
            }
        }
    }
    let children = std::array::from_fn(|i| Box::new(children[i].clone()));
    SparseVoxelOctree::Mixed { children }
}
