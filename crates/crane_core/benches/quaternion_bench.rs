use criterion::{criterion_group, criterion_main, Criterion};
use crane_core::math::{Quaternion, Vec3};
use std::hint::black_box;

fn bench_quat_multiply(c: &mut Criterion) {
    let q1 = Quaternion::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalized(), 1.5);
    let q2 = Quaternion::from_axis_angle(Vec3::new(4.0, 5.0, 6.0).normalized(), 0.7);
    
    c.bench_function("quat_multiply", |b| {
        b.iter(|| black_box(q1) * black_box(q2))
    });
}

fn bench_quat_rotate_vector(c: &mut Criterion) {
    let q = Quaternion::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalized(), 1.5);
    let v = Vec3::new(4.0, 5.0, 6.0);
    
    c.bench_function("quat_rotate_vector", |b| {
        b.iter(|| black_box(q).rotate_vector(black_box(v)))
    });
}

fn bench_quat_normalize(c: &mut Criterion) {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    
    c.bench_function("quat_normalize", |b| {
        b.iter(|| black_box(q).normalized())
    });
}

fn bench_quat_slerp(c: &mut Criterion) {
    let q1 = Quaternion::from_axis_angle(Vec3::Y, 0.0);
    let q2 = Quaternion::from_axis_angle(Vec3::Y, std::f64::consts::FRAC_PI_2);
    
    c.bench_function("quat_slerp", |b| {
        b.iter(|| black_box(q1).slerp(black_box(q2), black_box(0.5)))
    });
}

fn bench_quat_from_axis_angle(c: &mut Criterion) {
    let axis = Vec3::new(1.0, 2.0, 3.0).normalized();
    let angle = 1.5;
    
    c.bench_function("quat_from_axis_angle", |b| {
        b.iter(|| Quaternion::from_axis_angle(black_box(axis), black_box(angle)))
    });
}

fn bench_quat_to_axis_angle(c: &mut Criterion) {
    let q = Quaternion::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalized(), 1.5);
    
    c.bench_function("quat_to_axis_angle", |b| {
        b.iter(|| black_box(q).to_axis_angle())
    });
}

fn bench_quat_inverse(c: &mut Criterion) {
    let q = Quaternion::from_axis_angle(Vec3::new(1.0, 2.0, 3.0).normalized(), 1.5);
    
    c.bench_function("quat_inverse", |b| {
        b.iter(|| black_box(q).inverse())
    });
}

fn bench_quat_from_rotation_arc(c: &mut Criterion) {
    let from = Vec3::new(1.0, 0.0, 0.0);
    let to = Vec3::new(0.0, 1.0, 0.0);
    
    c.bench_function("quat_from_rotation_arc", |b| {
        b.iter(|| Quaternion::from_rotation_arc(black_box(from), black_box(to)))
    });
}

criterion_group!(
    benches,
    bench_quat_multiply,
    bench_quat_rotate_vector,
    bench_quat_normalize,
    bench_quat_slerp,
    bench_quat_from_axis_angle,
    bench_quat_to_axis_angle,
    bench_quat_inverse,
    bench_quat_from_rotation_arc,
);
criterion_main!(benches);
