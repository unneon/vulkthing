use crate::renderer::util::{Dev, StorageBuffer};
use crate::voxel::meshlet::{VoxelMesh, VoxelMeshlet};
use crate::voxel::vertex::VoxelVertex;
use nalgebra::Vector3;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct VoxelGpuMemory {
    meshlet_count: Arc<AtomicU32>,
    vertex_buffer: StorageBuffer<[VoxelVertex]>,
    vertex_count: usize,
    index_buffer: StorageBuffer<[u8]>,
    index_count: usize,
    meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
    chunk_size: usize,
    dev: Dev,
}

impl VoxelGpuMemory {
    pub fn new(
        meshlet_count: Arc<AtomicU32>,
        vertex_buffer: StorageBuffer<[VoxelVertex]>,
        index_buffer: StorageBuffer<[u8]>,
        meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
        chunk_size: usize,
        dev: Dev,
    ) -> VoxelGpuMemory {
        VoxelGpuMemory {
            meshlet_count,
            vertex_buffer,
            vertex_count: 0,
            index_buffer,
            index_count: 0,
            meshlet_buffer,
            chunk_size,
            dev,
        }
    }

    pub fn upload_meshlet(&mut self, chunk: Vector3<i64>, mut mesh: VoxelMesh) {
        let old_meshlet_count = self.meshlet_count.load(Ordering::SeqCst) as usize;
        let new_vertex_count = self.vertex_count + mesh.vertices.len();
        let new_index_count = self.index_count + mesh.indices.len();
        let new_meshlet_count = (old_meshlet_count as u32)
            .checked_add(mesh.meshlets.len() as u32)
            .unwrap() as usize;

        // The argument uses offsets local to the chunk mesh because the generation shouldn't deal
        // with the multithreading directly, so we need to fix them up now. Indices are local to the
        // meshlet, so they don't need to be fixed.
        for meshlet in &mut mesh.meshlets {
            meshlet.vertex_offset += self.vertex_count as u32;
            meshlet.index_offset += self.index_count as u32;
        }
        for vertex in &mut mesh.vertices {
            vertex.position += (chunk * self.chunk_size as i64).cast::<f32>();
        }

        let vertex_memory = &mut self.vertex_buffer.mapped()[self.vertex_count..new_vertex_count];
        MaybeUninit::write_slice(vertex_memory, &mesh.vertices);

        let index_memory = &mut self.index_buffer.mapped()[self.index_count..new_index_count];
        MaybeUninit::write_slice(index_memory, &mesh.indices);

        let meshlet_memory =
            &mut self.meshlet_buffer.mapped()[old_meshlet_count..new_meshlet_count];
        MaybeUninit::write_slice(meshlet_memory, &mesh.meshlets);

        self.vertex_count = new_vertex_count;
        self.index_count = new_index_count;
        self.meshlet_count
            .store(new_meshlet_count as u32, Ordering::SeqCst);
    }

    pub fn clear(&mut self) {
        // Holding the lock while updating the atomic is necessary, so leftover operations don't
        // mess up.
        self.vertex_count = 0;
        self.index_count = 0;
        self.meshlet_count.store(0, Ordering::SeqCst);
    }

    pub fn cleanup(&mut self) {
        self.vertex_buffer.cleanup(&self.dev);
        self.index_buffer.cleanup(&self.dev);
        self.meshlet_buffer.cleanup(&self.dev);
    }
}
