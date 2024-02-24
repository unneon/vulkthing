use crate::voxel::{ChunkPriorityAlgorithm, DIRECTIONS};
use nalgebra::Vector3;
use std::collections::HashSet;

pub struct SquareInvariant {
    camera: Vector3<i64>,
    loaded: HashSet<Vector3<i64>>,
    stable: Option<Stable>,
    queue: Vec<Vector3<i64>>,
    config: SquareInvariantConfig,
}

struct SquareInvariantConfig {
    render_distance_horizontal: usize,
    render_distance_vertical: usize,
    max_difference: usize,
}

struct Stable {
    min: Vector3<i64>,
    max: Vector3<i64>,
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
            stable: None,
            queue: Vec::new(),
            config: SquareInvariantConfig {
                render_distance_horizontal,
                render_distance_vertical,
                max_difference: 1,
            },
        }
    }
}

impl ChunkPriorityAlgorithm for SquareInvariant {
    fn select(&mut self) -> Option<Vector3<i64>> {
        if let Some(chunk) = self.queue.pop() {
            self.loaded.insert(chunk);
            if self.queue.is_empty() {
                if let Some(stable) = &mut self.stable {
                    stable.min = Vector3::new(
                        stable.min.x.min(chunk.x),
                        stable.min.y.min(chunk.y),
                        stable.min.z.min(chunk.z),
                    );
                    stable.max = Vector3::new(
                        stable.max.x.max(chunk.x),
                        stable.max.y.max(chunk.y),
                        stable.max.z.max(chunk.z),
                    );
                } else {
                    self.stable = Some(Stable {
                        min: chunk,
                        max: chunk,
                    });
                }
            }
            return Some(chunk);
        }

        let stable = self.stable.as_mut().unwrap();

        loop {
            let normal = DIRECTIONS
                .iter()
                .map(|direction| {
                    let is_positive = direction.sum() > 0;
                    let stable = if is_positive { stable.max } else { stable.min };
                    let absolute_direction = direction.abs();
                    let camera_coordinate = self.camera.component_mul(&absolute_direction).sum();
                    let stable_coordinate = stable.component_mul(&absolute_direction).sum();
                    let distance = camera_coordinate.abs_diff(stable_coordinate);
                    (distance, direction)
                })
                .max_by_key(|(distance, _)| *distance)
                .unwrap()
                .1;
            // TODO: The painful cube side wall iteration, reuse from earlier.
            return None;
        }
    }

    fn update_camera(&mut self, camera: Vector3<i64>) {
        self.camera = camera;
        if !self.loaded.contains(&camera) {
            self.stable = None;
            self.queue = vec![camera];
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
        self.stable = None;
        self.queue.clear();
        self.config.render_distance_horizontal = render_distance_horizontal;
        self.config.render_distance_vertical = render_distance_vertical;
    }
}
