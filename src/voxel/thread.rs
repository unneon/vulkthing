use crate::voxel::chunk_priority::ChunkPriorityAlgorithm;
use crate::voxel::meshing::generate_mesh;
use crate::voxel::neighbourhood::Neighbourhood;
use crate::voxel::world_generation::{generate_chunk_svo, generate_heightmap};
use crate::voxel::{meshlet, VoxelsShared};
use nalgebra::Vector3;
use std::sync::Arc;

pub fn voxel_thread(shared: &VoxelsShared) {
    let mut state = shared.state.lock().unwrap();
    loop {
        if state.shutdown {
            break;
        }

        let config = state.config.clone();
        let config_generation = state.config_generation;
        let noise = state.heightmap_noise.clone();

        state
            .chunk_priority
            .update_camera(*shared.camera.lock().unwrap());
        let Some(chunk) = state.chunk_priority.select() else {
            state = shared.wake.wait(state).unwrap();
            continue;
        };

        let mut svos = Vec::new();
        for oz in -1..=1 {
            for oy in -1..=1 {
                for ox in -1..=1 {
                    let offset = Vector3::new(ox, oy, oz);
                    let chunk = chunk + offset;
                    let svo = if let Some(svo) = state.loaded_svos.get(&chunk) {
                        svo.clone()
                    } else {
                        let heightmap =
                            if let Some(heightmap) = state.loaded_heightmaps.get(&chunk.xy()) {
                                heightmap.clone()
                            } else {
                                drop(state);
                                let heightmap =
                                    Arc::new(generate_heightmap(chunk.xy(), &noise, &config));
                                state = shared.state.lock().unwrap();
                                state
                                    .loaded_heightmaps
                                    .insert(chunk.xy(), heightmap.clone());
                                heightmap
                            };
                        drop(state);
                        let chunk_svo = Arc::new(generate_chunk_svo(chunk, &heightmap, &config));
                        state = shared.state.lock().unwrap();
                        state.loaded_svos.insert(chunk, chunk_svo.clone());
                        chunk_svo
                    };
                    svos.push(svo);
                }
            }
        }
        let neighbourhood = Neighbourhood::new(&svos, config.chunk_size as i64);
        drop(state);
        let raw_mesh = generate_mesh(neighbourhood, &config);
        let mut mesh = meshlet::from_unclustered_mesh(&raw_mesh);
        for meshlet in &mut mesh.meshlets {
            meshlet.chunk = chunk.try_cast::<i16>().unwrap();
        }
        state = shared.state.lock().unwrap();
        if config_generation != state.config_generation {
            continue;
        }
        state.gpu_memory.upload_meshlet(mesh);
    }
}
