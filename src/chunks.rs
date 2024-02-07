use crate::renderer::vertex::Vertex;
use crate::voxel::Voxels;
use nalgebra::Vector3;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Chunks<'a> {
    voxels: Voxels,
    // TODO: That is really not in the spirit of Rust safety.
    buffer: &'a mut [MaybeUninit<Vertex>],
    vertices: Arc<AtomicU64>,
}

impl<'a> Chunks<'a> {
    pub fn new(voxels: Voxels, buffer: &'a mut [MaybeUninit<Vertex>]) -> Chunks {
        Chunks {
            voxels,
            buffer,
            vertices: Arc::new(AtomicU64::new(0)),
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
    }

    pub fn shared(&self) -> Arc<AtomicU64> {
        self.vertices.clone()
    }
}
