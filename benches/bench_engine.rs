use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use minerust::coords::{BlockPos, LocalBlockPos};
use minerust::mesher::build_chunk_mesh;
use minerust::noise::NoiseGenerator;
use minerust::world::generate_chunk;

fn bench_chunk_generation(c: &mut Criterion) {
    let noise = NoiseGenerator::new(133742);
    c.bench_function("generate_chunk_procedural", |b| {
        b.iter(|| {
            generate_chunk(
                black_box(0),
                black_box(0),
                black_box(&noise),
                black_box(133742),
            )
        });
    });
}

fn bench_greedy_meshing(c: &mut Criterion) {
    let noise = NoiseGenerator::new(133742);
    let center = generate_chunk(0, 0, &noise, 133742);
    let north = generate_chunk(0, 1, &noise, 133742);
    let south = generate_chunk(0, -1, &noise, 133742);
    let east = generate_chunk(1, 0, &noise, 133742);
    let west = generate_chunk(-1, 0, &noise, 133742);

    let mut group = c.benchmark_group("chunk_mesher");
    group.bench_function("greedy_meshing_active", |b| {
        b.iter(|| {
            build_chunk_mesh(
                black_box(&center),
                black_box(Some(&north)),
                black_box(Some(&south)),
                black_box(Some(&east)),
                black_box(Some(&west)),
                black_box(true),
                black_box(true),
            )
        });
    });

    group.bench_function("naive_meshing_fallback", |b| {
        b.iter(|| {
            build_chunk_mesh(
                black_box(&center),
                black_box(Some(&north)),
                black_box(Some(&south)),
                black_box(Some(&east)),
                black_box(Some(&west)),
                black_box(true),
                black_box(false),
            )
        });
    });
    group.finish();
}

fn bench_coordinate_math(c: &mut Criterion) {
    c.bench_function("block_to_chunk_and_local_conversion", |b| {
        b.iter(|| {
            let pos = BlockPos::new(black_box(-12345), black_box(64), black_box(98765));
            let (chunk, local) = pos.to_chunk_and_local();
            let idx = local.to_index();
            let back = LocalBlockPos::from_index(idx).to_world(chunk);
            black_box(back);
        });
    });
}

criterion_group!(
    benches,
    bench_chunk_generation,
    bench_greedy_meshing,
    bench_coordinate_math,
);
criterion_main!(benches);
