use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Vector3;
use vulkthing::voxels::DIRECTIONS;

pub fn heightmap_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("voxel heightmap generate");
    let mut voxels = vulkthing::voxels::Voxels::new(1234, &mut []).0;
    group.significance_level(0.001);
    group.sample_size(5000);
    group.bench_function("noise", |b| {
        b.iter(|| {
            for z in -1..1 {
                for x in -4..4 {
                    for y in -4..4 {
                        let chunk = Vector3::new(x, y, z);
                        black_box(voxels.generate_heightmap_noise(black_box(chunk)));
                    }
                }
            }
        })
    });
    group.bench_function("bracket-noise", |b| {
        b.iter(|| {
            for z in -1..1 {
                for x in -4..4 {
                    for y in -4..4 {
                        let chunk = Vector3::new(x, y, z);
                        black_box(voxels.generate_heightmap_bracket_noise(black_box(chunk)));
                    }
                }
            }
        })
    });
    group.finish();
}

pub fn svo_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparse voxel octree generate");
    let mut voxels = vulkthing::voxels::Voxels::new(929, &mut []).0;
    let chunk = Vector3::new(0, 0, 0);
    let heightmap = voxels.generate_heightmap_bracket_noise(chunk);
    group.significance_level(0.001);
    group.sample_size(5000);
    group.bench_function("classic", |b| {
        b.iter(|| {
            for z in -2..2 {
                let chunk = Vector3::new(chunk.x, chunk.y, z);
                black_box(voxels.generate_chunk_svo(black_box(chunk), black_box(&heightmap)));
            }
        })
    });
    group.finish();
}

pub fn mesh_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("voxel mesh generate");
    let mut voxels = vulkthing::voxels::Voxels::new(919, &mut []).0;
    let chunk = Vector3::new(0, 0, 0);
    voxels.load_svo_cpu(chunk);
    for direction in DIRECTIONS {
        voxels.load_svo_cpu(chunk + direction);
    }
    let chunk_svo = &voxels.loaded_cpu[&chunk];
    let neighbour_svos = std::array::from_fn(|i| &voxels.loaded_cpu[&(chunk + DIRECTIONS[i])]);
    group.significance_level(0.001);
    group.sample_size(5000);
    group.bench_function("classic", |b| {
        b.iter(|| {
            black_box(voxels.generate_chunk_mesh(chunk, chunk_svo, neighbour_svos));
        })
    });
    group.finish();
}

criterion_group!(benches, heightmap_generate, svo_generate, mesh_generate);
criterion_main!(benches);
