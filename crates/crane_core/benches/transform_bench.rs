use criterion::{criterion_group, criterion_main, Criterion};
use crane_core::math::{Transform, Vec3, Quaternion};
use std::hint::black_box;

fn bench_transform_point(c: &mut Criterion) {
    let t = Transform::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quaternion::from_axis_angle(Vec3::Y, 1.5),
        2.0,
    );
    let point = Vec3::new(1.0, 2.0, 3.0);
    
    c.bench_function("transform_point", |b| {
        b.iter(|| black_box(t).transform_point(black_box(point)))
    });
}

fn bench_transform_vector(c: &mut Criterion) {
    let t = Transform::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quaternion::from_axis_angle(Vec3::Y, 1.5),
        2.0,
    );
    let vector = Vec3::new(1.0, 2.0, 3.0);
    
    c.bench_function("transform_vector", |b| {
        b.iter(|| black_box(t).transform_vector(black_box(vector)))
    });
}

fn bench_transform_combine(c: &mut Criterion) {
    let t1 = Transform::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quaternion::from_axis_angle(Vec3::Y, 1.5),
        2.0,
    );
    let t2 = Transform::new(
        Vec3::new(5.0, 10.0, 15.0),
        Quaternion::from_axis_angle(Vec3::X, 0.7),
        1.5,
    );
    
    c.bench_function("transform_combine", |b| {
        b.iter(|| black_box(t1).combine(black_box(t2)))
    });
}

fn bench_transform_inverse(c: &mut Criterion) {
    let t = Transform::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quaternion::from_axis_angle(Vec3::Y, 1.5),
        2.0,
    );
    
    c.bench_function("transform_inverse", |b| {
        b.iter(|| black_box(t).inverse())
    });
}

fn bench_transform_lerp(c: &mut Criterion) {
    let t1 = Transform::from_position(Vec3::ZERO);
    let t2 = Transform::from_position(Vec3::new(10.0, 10.0, 10.0));
    
    c.bench_function("transform_lerp", |b| {
        b.iter(|| black_box(t1).lerp(black_box(t2), black_box(0.5)))
    });
}

criterion_group!(
    benches,
    bench_transform_point,
    bench_transform_vector,
    bench_transform_combine,
    bench_transform_inverse,
    bench_transform_lerp,
);
criterion_main!(benches);
