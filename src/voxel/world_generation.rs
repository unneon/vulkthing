use crate::voxel::material::Material;
use crate::voxel::sparse_octree::SparseOctree;
use crate::voxel::VoxelsConfig;
use bracket_noise::prelude::FastNoise;
use nalgebra::{DMatrix, Vector2, Vector3};

pub fn generate_heightmap(
    chunk_column: Vector2<i64>,
    noise: &FastNoise,
    config: &VoxelsConfig,
) -> DMatrix<i64> {
    let chunk_coordinates = chunk_column * config.chunk_size as i64;
    let mut heightmap = DMatrix::from_element(config.chunk_size, config.chunk_size, 0);
    for x in 0..config.chunk_size {
        for y in 0..config.chunk_size {
            let column_coordinates = chunk_coordinates + Vector2::new(x as i64, y as i64);
            let noise_position = column_coordinates.cast::<f32>() * config.heightmap_frequency;
            let raw_noise = noise.get_noise(noise_position.x, noise_position.y);
            let scaled_noise = (raw_noise + config.heightmap_bias) * config.heightmap_amplitude;
            heightmap[(x, y)] = scaled_noise.round() as i64;
        }
    }
    heightmap
}

pub fn generate_chunk_svo(
    chunk: Vector3<i64>,
    heightmap: &DMatrix<i64>,
    config: &VoxelsConfig,
) -> SparseOctree {
    assert_eq!(heightmap.nrows(), config.chunk_size);
    assert_eq!(heightmap.ncols(), config.chunk_size);
    recursive_generate_svo(
        0,
        0,
        chunk.z * config.chunk_size as i64,
        config.chunk_size,
        heightmap,
    )
}

fn recursive_generate_svo(
    x: usize,
    y: usize,
    z: i64,
    n: usize,
    heightmap: &DMatrix<i64>,
) -> SparseOctree {
    'check_all_same: {
        let material = material_from_height(heightmap[(x, y)], z);
        for ly in y..y + n {
            for lx in x..x + n {
                let height = heightmap[(lx, ly)];
                let low_material = material_from_height(height, z);
                let high_material = material_from_height(height, z + n as i64 - 1);
                if low_material != material || high_material != material {
                    break 'check_all_same;
                }
            }
        }
        return SparseOctree::Uniform { kind: material };
    }
    let children = Box::new(std::array::from_fn(|index| {
        let dz = index / 4;
        let dy = index % 4 / 2;
        let dx = index % 2;
        recursive_generate_svo(
            x + dx * n / 2,
            y + dy * n / 2,
            z + dz as i64 * n as i64 / 2,
            n / 2,
            heightmap,
        )
    }));
    SparseOctree::Mixed { children }
}

fn material_from_height(height: i64, z: i64) -> Material {
    if height <= z {
        Material::Air
    } else if height <= z + 1 {
        Material::Grass
    } else if height <= z + 5 {
        Material::Dirt
    } else {
        Material::Stone
    }
}
