use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nalgebra::Vector3;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("voxel heightmap generation");
    let mut voxels = vulkthing::voxels::Voxels::new(&mut []).0;
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
