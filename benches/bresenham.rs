use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use torchbearer::bresenham::{BresenhamCircle, BresenhamLine, ThickBresenhamCircle};

const CIRCLE_RADIUS: i32 = 60;

pub fn bresenham_line(c: &mut Criterion) {
    c.benchmark_group("bresenham")
        .bench_function("line", |bencher| {
            bencher.iter(|| {
                let line = BresenhamLine::new(black_box((0, 0)), black_box((500, 200)));
                let _vec: Vec<_> = line.collect();
            });
        });
}

pub fn bresenham_circle(c: &mut Criterion) {
    c.benchmark_group("bresenham")
        .bench_function("circle", |b| {
            b.iter(|| {
                let circle = BresenhamCircle::new(black_box((0, 0)), black_box(CIRCLE_RADIUS));
                let _vec = circle.collect::<Vec<_>>();
            });
        });
}

pub fn thick_bresenham_circle(c: &mut Criterion) {
    c.benchmark_group("bresenham")
        .bench_function("thick_circle", |b| {
            b.iter(|| {
                let circle = ThickBresenhamCircle::new(black_box((0, 0)), black_box(CIRCLE_RADIUS));
                let _vec = circle.collect::<Vec<_>>();
            });
        });
}

criterion_group!(
    benches,
    bresenham_line,
    bresenham_circle,
    thick_bresenham_circle
);
criterion_main!(benches);
