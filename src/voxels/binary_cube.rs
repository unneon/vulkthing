use crate::voxels::DIRECTIONS;
use nalgebra::Vector3;

pub struct BinaryCube {
    pub position: Vector3<i64>,
    size: BinaryCubeSize,
}

struct BinaryCubeSize {
    length: usize,
}

impl BinaryCube {
    pub fn new_at_zero(length: usize) -> BinaryCube {
        BinaryCube {
            position: Vector3::new(0, 0, 0),
            size: BinaryCubeSize { length },
        }
    }

    pub fn subdivide(&self) -> impl Iterator<Item = BinaryCube> {
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

    pub fn is_single_voxel(&self) -> bool {
        self.size.length == 1
    }
}
