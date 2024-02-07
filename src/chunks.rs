use crate::renderer::lifecycle::create_vertex_buffer;
use crate::renderer::uniform::{Material, Transform};
use crate::renderer::util::{Dev, UniformBuffer};
use crate::renderer::FRAMES_IN_FLIGHT;
use crate::types::Chunk;
use crate::voxel::Voxels;
use ash::vk;
use nalgebra::Vector3;
use std::sync::{Arc, Mutex};

pub struct Chunks {
    shared: Arc<Mutex<ChunksShared>>,
    slot_count: usize,
    voxels: Voxels,
    dev: Dev,
}

pub struct ChunksShared {
    pub slots: Vec<ChunkSlot>,
    finished_frames: u64,
}

#[derive(Debug)]
pub struct ChunkSlot {
    pub active: bool,
    pub buffer: vk::Buffer,
    pub position: Vector3<f32>,
    pub triangle_count: u64,
    unused_after: u64,
}

impl Chunks {
    pub fn new(slot_count: usize, voxels: Voxels, dev: Dev) -> Chunks {
        let mut slots = Vec::new();
        for i in 0..slot_count {
            slots.push(ChunkSlot {
                active: false,
                buffer: vk::Buffer::null(),
                position: Vector3::zeros(),
                triangle_count: 0,
                unused_after: 0,
            });
        }
        let shared = Arc::new(Mutex::new(ChunksShared {
            slots,
            finished_frames: 0,
        }));
        Chunks {
            shared,
            slot_count,
            voxels,
            dev,
        }
    }

    pub fn generate_chunk(&self, chunk: Vector3<i64>) {
        let heightmap = self.voxels.generate_chunk_heightmap(chunk);
        let svo = self.voxels.generate_chunk_svo(chunk, &heightmap);
        let mesh = self.voxels.generate_chunk_mesh(&svo);
        if mesh.vertices.is_empty() {
            return;
        }
        let vertex_buffer = create_vertex_buffer(&mesh.vertices, false, &self.dev);
        let mut shared = self.shared.lock().unwrap();
        let slot = shared.slots.iter_mut().find(|slot| !slot.active).unwrap();
        slot.active = true;
        slot.buffer = vertex_buffer.buffer;
        slot.position = (chunk * (self.voxels.chunk_size as i64)).cast::<f32>();
        slot.triangle_count = mesh.vertices.len() as u64 / 3;
    }

    pub fn shared(&self) -> Arc<Mutex<ChunksShared>> {
        self.shared.clone()
    }
}
