use crate::config::{
    DEFAULT_VOXEL_CHUNK_SIZE, DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL,
    DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL,
};
use crate::renderer::vertex::Vertex;
use crate::voxel::Voxels;
use log::debug;
use nalgebra::Vector3;
use std::collections::HashSet;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Chunks<'a> {
    voxels: Voxels,
    // TODO: That is really not in the spirit of Rust safety.
    buffer: &'a mut [MaybeUninit<Vertex>],
    vertices: Arc<AtomicU64>,
    loaded: HashSet<Vector3<i64>>,
}

impl<'a> Chunks<'a> {
    pub fn new(voxels: Voxels, buffer: &'a mut [MaybeUninit<Vertex>]) -> Chunks {
        Chunks {
            voxels,
            buffer,
            vertices: Arc::new(AtomicU64::new(0)),
            loaded: HashSet::new(),
        }
    }

    pub fn update_camera(&mut self, camera: Vector3<f32>) {
        let camera_chunk = Vector3::new(
            (camera.x / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
            (camera.y / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
            (camera.z / DEFAULT_VOXEL_CHUNK_SIZE as f32).floor() as i64,
        );
        let distance_horizontal = DEFAULT_VOXEL_RENDER_DISTANCE_HORIZONTAL as i64;
        let distance_vertical = DEFAULT_VOXEL_RENDER_DISTANCE_VERTICAL as i64;
        let range_horizontal = -distance_horizontal..=distance_horizontal;
        let range_vertical = -distance_vertical..=distance_vertical;
        'outer: for dx in range_horizontal.clone() {
            for dy in range_horizontal.clone() {
                for dz in range_vertical.clone() {
                    let chunk = camera_chunk + Vector3::new(dx, dy, dz);
                    if !self.loaded.contains(&chunk) {
                        self.generate_chunk(chunk);
                        self.loaded.insert(chunk);
                        break 'outer;
                    }
                }
            }
        }
    }

    pub fn generate_chunk(&mut self, chunk: Vector3<i64>) {
        let heightmap = self.voxels.generate_chunk_heightmap(chunk);
        let svo = self.voxels.generate_chunk_svo(chunk, &heightmap);
        let mesh = self.voxels.generate_chunk_mesh(chunk, &svo);
        if mesh.vertices.is_empty() {
            return;
        }
        let old_vertex_count = self.vertices.load(Ordering::SeqCst) as usize;
        let new_vertex_count = old_vertex_count + mesh.vertices.len();
        let memory = &mut self.buffer[old_vertex_count..new_vertex_count];
        MaybeUninit::write_slice(memory, &mesh.vertices);
        self.vertices
            .store(new_vertex_count as u64, Ordering::SeqCst);
        debug!(
            "generated chunk, \x1B[1mid\x1B[0m: {},{},{}, \x1B[1mvertices:\x1B[0m: {}",
            chunk.x,
            chunk.y,
            chunk.z,
            mesh.vertices.len()
        );
    }

    pub fn shared(&self) -> Arc<AtomicU64> {
        self.vertices.clone()
    }
}
