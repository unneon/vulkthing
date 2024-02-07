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

pub struct BinaryCube {
    position: Vector3<i64>,
    size: BinaryCubeSize,
}

struct BinaryCubeSize {
    length: usize,
}

const DIRECTIONS: [Vector3<i64>; 6] = [
    Vector3::new(1, 0, 0),
    Vector3::new(-1, 0, 0),
    Vector3::new(0, 1, 0),
    Vector3::new(0, -1, 0),
    Vector3::new(0, 0, 1),
    Vector3::new(0, 0, -1),
];

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
            }
        }
        heightmap
    }

    pub fn generate_chunk_svo(
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

    pub fn generate_chunk_mesh(&self, voxels: &SparseVoxelOctree) -> MeshData {
        let cube = BinaryCube::new_at_zero(self.chunk_size);
        let mut vertices = Vec::new();
        self.generate_chunk_mesh_impl(cube, voxels, voxels, &mut vertices);
        MeshData { vertices }
    }

    fn generate_chunk_mesh_impl(
        &self,
        cube: BinaryCube,
        cube_voxels: &SparseVoxelOctree,
        root_voxels: &SparseVoxelOctree,
        vertices: &mut Vec<Vertex>,
    ) {
        if cube.is_single_voxel() {
            self.generate_chunk_mesh_voxel(cube.position, root_voxels, vertices);
            return;
        }

        match cube_voxels {
            SparseVoxelOctree::Uniform { .. } => {
                for side_voxel in cube.side_voxels() {
                    self.generate_chunk_mesh_voxel(side_voxel, root_voxels, vertices);
                }
            }
            SparseVoxelOctree::Mixed { children } => {
                for (sub_cube, child) in cube.subdivide().zip(children.iter()) {
                    self.generate_chunk_mesh_impl(sub_cube, child, root_voxels, vertices);
                }
            }
        };
    }

    fn generate_chunk_mesh_voxel(
        &self,
        position: Vector3<i64>,
        root_voxels: &SparseVoxelOctree,
        vertices: &mut Vec<Vertex>,
    ) {
        for direction in DIRECTIONS {
            let side = self.generate_chunk_mesh_side(position, direction, root_voxels);
            if let Some(side) = side {
                vertices.extend_from_slice(&side);
            }
        }
    }

    fn generate_chunk_mesh_side(
        &self,
        position: Vector3<i64>,
        normal: Vector3<i64>,
        voxels: &SparseVoxelOctree,
    ) -> Option<[Vertex; 6]> {
        if !voxels.at(position, self.chunk_size as i64) {
            return None;
        }
        let neighbour = position + normal;
        let to_out_of_bounds = (neighbour.x < 0 || neighbour.x >= self.chunk_size as i64)
            || (neighbour.y < 0 || neighbour.y >= self.chunk_size as i64)
            || (neighbour.z < 0 || neighbour.z >= self.chunk_size as i64);
        if !to_out_of_bounds && voxels.at(neighbour, self.chunk_size as i64) {
            return None;
        }
        let rot1 = Vector3::new(normal.z.abs(), normal.x.abs(), normal.y.abs());
        let rot2 = Vector3::new(normal.y.abs(), normal.z.abs(), normal.x.abs());
        let base = if normal.x + normal.y + normal.z > 0 {
            position + normal
        } else {
            position
        };
        let (rot1, rot2) = if normal == rot1.cross(&rot2) {
            (rot1, rot2)
        } else {
            (rot2, rot1)
        };
        let v1 = Vertex {
            position: base.cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v2 = Vertex {
            position: (base + rot1).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v3 = Vertex {
            position: (base + rot2).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        let v4 = Vertex {
            position: (base + rot1 + rot2).cast::<f32>(),
            normal: normal.cast::<f32>(),
        };
        Some([v1, v2, v3, v2, v4, v3])
    }
}

impl SparseVoxelOctree {
    pub fn at(&self, point: Vector3<i64>, local_size: i64) -> bool {
        match self {
            SparseVoxelOctree::Uniform { kind } => *kind == VoxelKind::Stone,
            SparseVoxelOctree::Mixed { children } => {
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

impl BinaryCube {
    pub fn new_at_zero(length: usize) -> BinaryCube {
        BinaryCube {
            position: Vector3::new(0, 0, 0),
            size: BinaryCubeSize { length },
        }
    }

    fn subdivide(&self) -> impl Iterator<Item = BinaryCube> {
        let position = self.position;
        let sublength = self.size.length / 2;
        (0..2).flat_map(move |dz| {
            (0..2).flat_map(move |dy| {
                (0..2).map(move |dx| BinaryCube {
                    position: position + sublength as i64 * Vector3::new(dx, dy, dz),
                    size: BinaryCubeSize { length: sublength },
                })
            })
        })
    }

    pub fn side_voxels(&self) -> impl Iterator<Item = Vector3<i64>> {
        let position = self.position;
        let length = self.size.length as i64;
        DIRECTIONS.iter().flat_map(move |direction| {
            let du = if direction.x == 0 {
                Vector3::new(1, 0, 0)
            } else {
                Vector3::new(0, 1, 0)
            };
            let dv = if direction.z == 0 {
                Vector3::new(0, 0, 1)
            } else {
                Vector3::new(0, 1, 0)
            };
            let side_base = position
                + Vector3::new(
                    if direction.x > 0 { length - 1 } else { 0 },
                    if direction.y > 0 { length - 1 } else { 0 },
                    if direction.z > 0 { length - 1 } else { 0 },
                );
            (0..length).flat_map(move |i| (0..length).map(move |j| side_base + i * du + j * dv))
        })
    }

    fn is_single_voxel(&self) -> bool {
        self.size.length == 1
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
