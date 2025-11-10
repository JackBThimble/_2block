use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use crane_core::math::Vec3;

fn bench_dot_product(c: &mut Criterion) {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_dot", |b| {
        b.iter(|| {
            black_box(v1).dot(black_box(v2))
        })
    });
}

fn bench_cross_product(c: &mut Criterion) {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_cross", |b| {
        b.iter(|| {
            black_box(v1).cross(black_box(v2))
        })
    });
}

fn bench_normalize(c: &mut Criterion) {
    let v = Vec3::new(3.0, 4.0, 5.0);

    c.bench_function("vec3_normalize", |b| {
        b.iter(|| {
            black_box(v).normalized()
        })
    });
}

fn bench_add(c: &mut Criterion) {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_add", |b| {
        b.iter(|| {
            black_box(v1) + black_box(v2)
        })
    });
}

fn bench_lerp(c: &mut Criterion) {
    let v1 = Vec3::new(0.0, 0.0, 0.0);
    let v2 = Vec3::new(10.0, 10.0, 10.0);

    c.bench_function("vec3_lerp", |b| {
        b.iter(|| {
            black_box(v1).lerp(black_box(v2), black_box(0.5))
        })
    });
}

fn bench_distance(c: &mut Criterion) {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    c.bench_function("vec3_distance", |b| {
        b.iter(|| {
            black_box(v1).distance(black_box(v2))
        })
    });
}

criterion_group!(
benches,
    bench_dot_product,
    bench_cross_product,
    bench_normalize,
    bench_add,
    bench_lerp,
    bench_distance
);

criterion_main!(benches);
