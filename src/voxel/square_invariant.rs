use crate::util::geometry::Cuboid;
use crate::voxel::{ChunkPriorityAlgorithm, DIRECTIONS};
use nalgebra::Vector3;
use std::collections::HashSet;

pub struct SquareInvariant {
    camera: Vector3<i64>,
    loaded: HashSet<Vector3<i64>>,
    stable: Cuboid<i64>,
    queue: Vec<Vector3<i64>>,
    config: SquareInvariantConfig,
}

struct SquareInvariantConfig {
    render_distance_horizontal: usize,
    render_distance_vertical: usize,
}

impl SquareInvariant {
    pub fn new(
        camera: Vector3<i64>,
        render_distance_horizontal: usize,
        render_distance_vertical: usize,
    ) -> SquareInvariant {
        SquareInvariant {
            camera,
            loaded: HashSet::new(),
            stable: Cuboid::new_empty(),
            queue: Vec::new(),
            config: SquareInvariantConfig {
                render_distance_horizontal,
                render_distance_vertical,
            },
        }
    }
}

impl ChunkPriorityAlgorithm for SquareInvariant {
    fn select(&mut self) -> Option<Vector3<i64>> {
        if let Some(chunk) = self.queue.pop() {
            self.loaded.insert(chunk);
            return Some(chunk);
        }

        if self.stable.is_empty() {
            self.stable = Cuboid::new_unit_cube(self.camera);
            if self.loaded.insert(self.camera) {
                return Some(self.camera);
            }
        }

        assert!(self.stable.contains(self.camera));
        loop {
            let normal = closest_side(self.camera, self.stable);
            self.stable = self.stable.extend_in_direction(normal);
            for voxel in self.stable.side_voxels(normal) {
                if !self.loaded.contains(&voxel) {
                    self.queue.push(voxel);
                }
            }
            if let Some(chunk) = self.queue.pop() {
                self.loaded.insert(chunk);
                break Some(chunk);
            }
        }
    }

    fn update_camera(&mut self, camera: Vector3<i64>) {
        self.camera = camera;
        if !self.stable.contains(camera) {
            self.stable = Cuboid::new_empty();
            self.queue.clear();
        }
    }

    fn clear(
        &mut self,
        camera: Vector3<i64>,
        render_distance_horizontal: usize,
        render_distance_vertical: usize,
    ) {
        self.camera = camera;
        self.loaded.clear();
        self.stable = Cuboid::new_empty();
        self.queue.clear();
        self.config.render_distance_horizontal = render_distance_horizontal;
        self.config.render_distance_vertical = render_distance_vertical;
    }
}

fn closest_side(camera: Vector3<i64>, stable: Cuboid<i64>) -> Vector3<i64> {
    DIRECTIONS
        .iter()
        .map(|&direction| {
            let distance = stable.distance_from_inside(camera, direction);
            (distance, direction)
        })
        .min_by_key(|(distance, _)| *distance)
        .unwrap()
        .1
}
