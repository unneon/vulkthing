use crate::renderer::util::{Dev, StorageBuffer};
use crate::voxel::gpu::VoxelGpuMemory;
use crate::voxel::local_mesh::LocalMesh;
use crate::voxel::meshlet;
use crate::voxel::meshlet::{VoxelMesh, VoxelMeshlet, VoxelTriangle, VoxelVertex};
use nalgebra::Vector3;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub struct VoxelMeshletMemory {
    meshlet_count: Arc<AtomicU32>,
    vertex_buffer: StorageBuffer<[VoxelVertex]>,
    vertex_count: usize,
    triangle_buffer: StorageBuffer<[VoxelTriangle]>,
    triangle_count: usize,
    meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
    dev: Dev,
}

impl VoxelMeshletMemory {
    pub fn new(
        meshlet_count: Arc<AtomicU32>,
        vertex_buffer: StorageBuffer<[VoxelVertex]>,
        triangle_buffer: StorageBuffer<[VoxelTriangle]>,
        meshlet_buffer: StorageBuffer<[VoxelMeshlet]>,
        dev: Dev,
    ) -> VoxelMeshletMemory {
        VoxelMeshletMemory {
            meshlet_count,
            vertex_buffer,
            vertex_count: 0,
            triangle_buffer,
            triangle_count: 0,
            meshlet_buffer,
            dev,
        }
    }
}

impl VoxelGpuMemory for VoxelMeshletMemory {
    fn prepare_func(&self) -> fn(LocalMesh, Vector3<i64>) -> Box<dyn std::any::Any> {
        |mesh, chunk| Box::new(prepare(mesh, chunk))
    }

    fn upload(&mut self, mesh: Box<dyn std::any::Any>) {
        let mut mesh = mesh.downcast::<VoxelMesh>().unwrap();
        let old_meshlet_count = self.meshlet_count.load(Ordering::SeqCst) as usize;
        let new_vertex_count = self.vertex_count + mesh.vertices.len();
        let new_triangle_count = self.triangle_count + mesh.triangles.len();
        let new_meshlet_count = (old_meshlet_count as u32)
            .checked_add(mesh.meshlets.len() as u32)
            .unwrap() as usize;

        // The argument uses offsets local to the chunk mesh because the generation shouldn't deal
        // with the multithreading directly, so we need to fix them up now. Indices are local to the
        // meshlet, so they don't need to be fixed.
        for meshlet in &mut mesh.meshlets {
            meshlet.vertex_offset += self.vertex_count as u32;
            meshlet.triangle_offset += self.triangle_count as u32;
        }

        let vertex_memory = &mut self.vertex_buffer.mapped()[self.vertex_count..new_vertex_count];
        MaybeUninit::copy_from_slice(vertex_memory, &mesh.vertices);

        let triangle_memory =
            &mut self.triangle_buffer.mapped()[self.triangle_count..new_triangle_count];
        MaybeUninit::copy_from_slice(triangle_memory, &mesh.triangles);

        let meshlet_memory =
            &mut self.meshlet_buffer.mapped()[old_meshlet_count..new_meshlet_count];
        MaybeUninit::copy_from_slice(meshlet_memory, &mesh.meshlets);

        self.vertex_count = new_vertex_count;
        self.triangle_count = new_triangle_count;
        self.meshlet_count
            .store(new_meshlet_count as u32, Ordering::SeqCst);
    }

    fn clear(&mut self) {
        // Holding the lock while updating the atomic is necessary, so leftover operations don't
        // mess up.
        self.vertex_count = 0;
        self.triangle_count = 0;
        self.meshlet_count.store(0, Ordering::SeqCst);
    }

    fn cleanup(&mut self) {
        self.vertex_buffer.cleanup(&self.dev);
        self.triangle_buffer.cleanup(&self.dev);
        self.meshlet_buffer.cleanup(&self.dev);
    }
}

fn prepare(raw_mesh: LocalMesh, chunk: Vector3<i64>) -> VoxelMesh {
    let mut mesh = meshlet::from_unclustered_mesh(&raw_mesh);
    for meshlet in &mut mesh.meshlets {
        meshlet.chunk = chunk.try_cast::<i16>().unwrap();
    }
    mesh
}
