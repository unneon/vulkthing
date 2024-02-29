use crate::util::geometry::Cuboid;
use crate::voxel::DIRECTIONS;
use nalgebra::Vector3;
use std::collections::HashSet;

pub trait ChunkPriorityAlgorithm {
    fn select(&mut self) -> Option<Vector3<i64>>;

    fn update_camera(&mut self, camera: Vector3<i64>);

    fn clear(
        &mut self,
        camera: Vector3<i64>,
        render_distance_horizontal: i64,
        render_distance_vertical: i64,
    );
}

pub struct ChunkPriority {
    camera: Vector3<i64>,
    loaded: HashSet<Vector3<i64>>,
    stable: Cuboid<i64>,
    queue: Vec<Vector3<i64>>,
    config: Config,
}

struct Config {
    render_distance_horizontal: i64,
    render_distance_vertical: i64,
}

impl ChunkPriority {
    pub fn new(
        camera: Vector3<i64>,
        render_distance_horizontal: i64,
        render_distance_vertical: i64,
    ) -> ChunkPriority {
        ChunkPriority {
            camera,
            loaded: HashSet::new(),
            stable: Cuboid::new_empty(),
            queue: Vec::new(),
            config: Config {
                render_distance_horizontal,
                render_distance_vertical,
            },
        }
    }

    fn closest_side(&self) -> Option<Vector3<i64>> {
        DIRECTIONS
            .iter()
            .filter_map(|&direction| {
                let distance = self.stable.distance_from_inside(self.camera, direction);
                if direction.z == 0 && distance > self.config.render_distance_horizontal {
                    return None;
                }
                if direction.z != 0 && distance > self.config.render_distance_vertical {
                    return None;
                }
                Some((distance, direction))
            })
            .min_by_key(|(distance, _)| *distance)
            .map(|(_, normal)| normal)
    }
}

impl ChunkPriorityAlgorithm for ChunkPriority {
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
            let Some(normal) = self.closest_side() else {
                break None;
            };
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
        render_distance_horizontal: i64,
        render_distance_vertical: i64,
    ) {
        self.camera = camera;
        self.loaded.clear();
        self.stable = Cuboid::new_empty();
        self.queue.clear();
        self.config.render_distance_horizontal = render_distance_horizontal;
        self.config.render_distance_vertical = render_distance_vertical;
    }
}
