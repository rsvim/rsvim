use assert_fs::prelude::*;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rsvim_core::evloop::EventLoop;
use rsvim_core::evloop::mock::*;
use rsvim_core::js::JsRuntimeForSnapshot;

fn create_snapshot(tp: &TempConfigDir) -> Vec<u8> {
  let snapshot_file = tp.xdg_data_home.child("snapshot.bin");

  // Prepare snapshot data
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  std::fs::write(snapshot_file.path(), vec.into_boxed_slice()).unwrap();
  std::fs::read(snapshot_file.path()).unwrap()
}

pub fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("startup time with snapshot", |b| {
    b.iter(|| fibonacci(black_box(20)))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
