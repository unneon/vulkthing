use crate::mesh::MeshData;
use crate::renderer::vertex::Vertex;
use crate::voxels::sparse_octree::SparseOctree;
use crate::voxels::{MeshingAlgorithm, VoxelKind, DIRECTIONS};
use nalgebra::{Vector2, Vector3};

pub struct GreedyMeshing;

struct State<'a> {
    chunk_size: usize,
    chunk_svo: &'a SparseOctree,
    neighbour_svos: [&'a SparseOctree; 6],
    slice_right: Vector3<i64>,
    slice_down: Vector3<i64>,
    slice_normal: Vector3<i64>,
    slice_normal_index: usize,
    slice_minus_normal_index: usize,
    slice_offset: i64,
    slice_used: Vec<bool>,
    vertices: Vec<Vertex>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum WallNormal {
    AlongSliceNormal,
    AlongMinusSliceNormal,
}

impl MeshingAlgorithm for GreedyMeshing {
    fn mesh(
        chunk_svo: &SparseOctree,
        neighbour_svos: [&SparseOctree; 6],
        chunk_size: usize,
    ) -> MeshData {
        let mut state = State {
            chunk_size,
            chunk_svo,
            neighbour_svos,
            slice_right: Vector3::zeros(),
            slice_down: Vector3::zeros(),
            slice_normal: Vector3::zeros(),
            slice_normal_index: 0,
            slice_minus_normal_index: 0,
            slice_offset: 0,
            slice_used: vec![false; chunk_size * chunk_size],
            vertices: Vec::new(),
        };
        let dx = Vector3::new(1, 0, 0);
        let dy = Vector3::new(0, 1, 0);
        let dz = Vector3::new(0, 0, 1);
        state.mesh_orientation(dx, dy, dz, 4, 5);
        state.mesh_orientation(dx, dz, dy, 2, 3);
        state.mesh_orientation(dy, dz, dx, 0, 1);
        MeshData {
            vertices: state.vertices,
        }
    }
}

impl State<'_> {
    fn mesh_orientation(
        &mut self,
        right: Vector3<i64>,
        down: Vector3<i64>,
        normal: Vector3<i64>,
        normal_index: usize,
        minus_normal_index: usize,
    ) {
        assert_eq!(DIRECTIONS[normal_index], normal);
        assert_eq!(DIRECTIONS[minus_normal_index], -normal);
        self.slice_right = right;
        self.slice_down = down;
        self.slice_normal = normal;
        self.slice_normal_index = normal_index;
        self.slice_minus_normal_index = minus_normal_index;
        for offset in 0..=self.chunk_size as i64 {
            self.slice_offset = offset;
            self.slice_used.fill(false);
            self.mesh_slice();
        }
    }

    fn mesh_slice(&mut self) {
        for y1 in 0..self.chunk_size as i64 {
            for x1 in 0..self.chunk_size as i64 {
                let Some(along) = self.wall(Vector2::new(x1, y1)) else {
                    continue;
                };

                let mut x2 = x1 + 1;
                while self.wall(Vector2::new(x2, y1)) == Some(along) {
                    x2 += 1;
                }

                let mut y2 = y1 + 1;
                while (x1..x2).all(|x| self.wall(Vector2::new(x, y2)) == Some(along)) {
                    y2 += 1;
                }

                for y in y1..y2 {
                    for x in x1..x2 {
                        self.slice_used[y as usize * self.chunk_size + x as usize] = true;
                    }
                }

                let top_left = self.convert_2d_to_3d(Vector2::new(x1, y1)).cast::<f32>();
                let top_right = self.convert_2d_to_3d(Vector2::new(x2, y1)).cast::<f32>();
                let bottom_left = self.convert_2d_to_3d(Vector2::new(x1, y2)).cast::<f32>();
                let bottom_right = self.convert_2d_to_3d(Vector2::new(x2, y2)).cast::<f32>();
                let normal = match along {
                    WallNormal::AlongSliceNormal => self.slice_normal,
                    WallNormal::AlongMinusSliceNormal => -self.slice_normal,
                }
                .cast::<f32>();
                let v1 = Vertex {
                    position: top_left,
                    normal,
                };
                let v2 = Vertex {
                    position: top_right,
                    normal,
                };
                let v3 = Vertex {
                    position: bottom_left,
                    normal,
                };
                let v4 = Vertex {
                    position: bottom_right,
                    normal,
                };
                let (v2, v3) = if self.slice_right.cross(&self.slice_down) == self.slice_normal {
                    (v2, v3)
                } else {
                    (v3, v2)
                };
                self.vertices.extend_from_slice(&[v1, v2, v3, v2, v4, v3]);
            }
        }
    }

    /// Checks whether a wall should be placed between a voxel position and a voxel a minus normal apart from it. Also
    /// checks the desired orientation of the wall.
    fn wall(&self, voxel_2d: Vector2<i64>) -> Option<WallNormal> {
        // Note this assert and the following condition refer to 2D coordinates, not 3D. The out of bounds checks later
        // are related only to the normal axis, so the only reason 2D coordinates would be out of bounds is because of
        // the closed-open interval convention used in mesh_slice function.
        assert!(voxel_2d.x >= 0 && voxel_2d.y >= 0);
        if voxel_2d.x >= self.chunk_size as i64 || voxel_2d.y >= self.chunk_size as i64 {
            return None;
        }
        if self.slice_used[voxel_2d.y as usize * self.chunk_size + voxel_2d.x as usize] {
            return None;
        }

        // We should never call this from the direction of negative neighbours (see loop in mesh_orientation). Positive
        // neighbours and normal leading into negative neighbours are handled in the if statements below.
        let voxel_3d = self.convert_2d_to_3d(voxel_2d);
        let voxel_kind = if !self.out_of_bounds_positive(voxel_3d) {
            self.chunk_svo.at(voxel_3d, self.chunk_size as i64)
        } else {
            let voxel_3d_in_neighbour = self.wrap_out_of_bounds(voxel_3d);
            self.neighbour_svos[self.slice_normal_index]
                .at(voxel_3d_in_neighbour, self.chunk_size as i64)
        };

        let neighbour_3d = voxel_3d - self.slice_normal;
        let neighbour_kind = if !self.out_of_bounds_negative(neighbour_3d) {
            self.chunk_svo.at(neighbour_3d, self.chunk_size as i64)
        } else {
            let neighbour_3d_in_neighbour = self.wrap_out_of_bounds(neighbour_3d);
            self.neighbour_svos[self.slice_minus_normal_index]
                .at(neighbour_3d_in_neighbour, self.chunk_size as i64)
        };
        // If the checked voxel is outside the chunk, the wall shouldn't be generated along the minus slice normal,
        // because it would belong to the other chunk. If the neighbour voxel is outside the chunk, the wall also
        // shouldn't be generated along the slice normal for the same reason.
        if voxel_kind == VoxelKind::Stone
            && neighbour_kind == VoxelKind::Air
            && !self.out_of_bounds_positive(voxel_3d)
        {
            Some(WallNormal::AlongMinusSliceNormal)
        } else if voxel_kind == VoxelKind::Air
            && neighbour_kind == VoxelKind::Stone
            && !self.out_of_bounds_negative(neighbour_3d)
        {
            Some(WallNormal::AlongSliceNormal)
        } else {
            None
        }
    }

    fn convert_2d_to_3d(&self, voxel: Vector2<i64>) -> Vector3<i64> {
        self.slice_offset * self.slice_normal
            + voxel.y * self.slice_down
            + voxel.x * self.slice_right
    }

    fn out_of_bounds_positive(&self, voxel: Vector3<i64>) -> bool {
        voxel.x >= self.chunk_size as i64
            || voxel.y >= self.chunk_size as i64
            || voxel.z >= self.chunk_size as i64
    }

    fn out_of_bounds_negative(&self, voxel: Vector3<i64>) -> bool {
        voxel.x < 0 || voxel.y < 0 || voxel.z < 0
    }

    fn wrap_out_of_bounds(&self, voxel: Vector3<i64>) -> Vector3<i64> {
        let positive = voxel.add_scalar(self.chunk_size as i64);
        Vector3::new(
            positive.x % self.chunk_size as i64,
            positive.y % self.chunk_size as i64,
            positive.z % self.chunk_size as i64,
        )
    }
}

#[test]
fn single_voxel_air_around_air() {
    let actual = GreedyMeshing::mesh(
        &SparseOctree::Uniform {
            kind: VoxelKind::Air,
        },
        [&SparseOctree::Uniform {
            kind: VoxelKind::Air,
        }; 6],
        1,
    );
    assert!(actual.vertices.is_empty());
}

#[test]
fn single_voxel_stone_around_stone() {
    let actual = GreedyMeshing::mesh(
        &SparseOctree::Uniform {
            kind: VoxelKind::Stone,
        },
        [&SparseOctree::Uniform {
            kind: VoxelKind::Stone,
        }; 6],
        1,
    );
    assert!(actual.vertices.is_empty());
}

#[test]
fn chunk16_air_around_air() {
    let actual = GreedyMeshing::mesh(
        &SparseOctree::Uniform {
            kind: VoxelKind::Air,
        },
        [&SparseOctree::Uniform {
            kind: VoxelKind::Air,
        }; 6],
        16,
    );
    assert!(actual.vertices.is_empty());
}

#[test]
fn chunk16_stone_around_stone() {
    let actual = GreedyMeshing::mesh(
        &SparseOctree::Uniform {
            kind: VoxelKind::Stone,
        },
        [&SparseOctree::Uniform {
            kind: VoxelKind::Stone,
        }; 6],
        16,
    );
    assert!(actual.vertices.is_empty());
}
