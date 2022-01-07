use criterion::{criterion_group, criterion_main, Criterion};
use torchbearer::bresenham::BresenhamLine;

pub fn bresenham_line(c: &mut Criterion) {
    c.bench_function("bresenham_line", |bencher| {
        bencher.iter(|| {
            let line = BresenhamLine::new((0, 0), (500, 200));
            let _vec: Vec<_> = line.collect();
        });
    });
}

criterion_group!(benches, bresenham_line);
criterion_main!(benches);
