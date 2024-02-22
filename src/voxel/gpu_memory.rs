use crate::renderer::util::{Dev, StorageBuffer};
use crate::voxel::meshlet::{VoxelMesh, VoxelMeshlet};
use crate::voxel::vertex::VoxelVertex;
use nalgebra::Vector3;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

pub struct VoxelGpuMemory {
    meshlet_count: AtomicU32,
    upload_state: Mutex<UploadState>,
}

struct UploadState {
    vertex_buffer: StorageBuffer<[VoxelVertex]>,
    vertex_count: usize,
    index_buffer: StorageBuffer<[u8]>,
    index_count: usize,
    meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
    chunk_size: usize,
}

impl VoxelGpuMemory {
    pub fn new(
        vertex_buffer: StorageBuffer<[VoxelVertex]>,
        index_buffer: StorageBuffer<[u8]>,
        meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
        chunk_size: usize,
    ) -> VoxelGpuMemory {
        VoxelGpuMemory {
            meshlet_count: AtomicU32::new(0),
            upload_state: Mutex::new(UploadState {
                vertex_buffer,
                vertex_count: 0,
                index_buffer,
                index_count: 0,
                meshlet_buffer,
                chunk_size,
            }),
        }
    }

    pub fn meshlet_count(&self) -> u32 {
        self.meshlet_count.load(Ordering::SeqCst)
    }

    pub fn upload_meshlet(&self, chunk: Vector3<i64>, mut mesh: VoxelMesh) {
        // This lock is held for the entire upload, but these are are fast (tens of microseconds) in
        // practice. The renderer uses the atomic so it doesn't wait either.
        let mut upload_state = self.upload_state.lock().unwrap();

        let old_vertex_count = upload_state.vertex_count;
        let old_index_count = upload_state.index_count;
        let old_meshlet_count = self.meshlet_count.load(Ordering::SeqCst) as usize;
        let new_vertex_count = old_vertex_count + mesh.vertices.len();
        let new_index_count = old_index_count + mesh.indices.len();
        let new_meshlet_count = (old_meshlet_count as u32)
            .checked_add(mesh.meshlets.len() as u32)
            .unwrap() as usize;

        // The argument uses offsets local to the chunk mesh because the generation shouldn't deal
        // with the multithreading directly, so we need to fix them up now. Indices are local to the
        // meshlet, so they don't need to be fixed.
        for meshlet in &mut mesh.meshlets {
            meshlet.vertex_offset += old_vertex_count as u32;
            meshlet.index_offset += old_index_count as u32;
        }
        for vertex in &mut mesh.vertices {
            vertex.position += (chunk * upload_state.chunk_size as i64).cast::<f32>();
        }

        let vertex_memory =
            &mut upload_state.vertex_buffer.mapped()[old_vertex_count..new_vertex_count];
        MaybeUninit::write_slice(vertex_memory, &mesh.vertices);

        let index_memory =
            &mut upload_state.index_buffer.mapped()[old_index_count..new_index_count];
        MaybeUninit::write_slice(index_memory, &mesh.indices);

        let meshlet_memory =
            &mut upload_state.meshlet_buffer.mapped()[old_meshlet_count..new_meshlet_count];
        MaybeUninit::write_slice(meshlet_memory, &mesh.meshlets);

        upload_state.vertex_count = new_vertex_count;
        upload_state.index_count = new_index_count;
        self.meshlet_count
            .store(new_meshlet_count as u32, Ordering::SeqCst);
    }

    pub fn clear(&self) {
        // Holding the lock while updating the atomic is necessary, so leftover operations don't
        // mess up.
        let mut upload_state = self.upload_state.lock().unwrap();
        upload_state.vertex_count = 0;
        upload_state.index_count = 0;
        self.meshlet_count.store(0, Ordering::SeqCst);
    }

    pub fn cleanup(&self, dev: &Dev) {
        let upload_state = self.upload_state.lock().unwrap();
        upload_state.vertex_buffer.cleanup(dev);
        upload_state.index_buffer.cleanup(dev);
        upload_state.meshlet_buffer.cleanup(dev);
    }
}
