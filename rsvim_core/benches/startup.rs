use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rsvim_core::evloop::EventLoop;

pub fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("startup time with snapshot", |b| {
    b.iter(|| fibonacci(black_box(20)))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
