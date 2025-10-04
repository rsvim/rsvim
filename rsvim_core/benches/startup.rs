#![allow(deprecated)]

use assert_fs::prelude::*;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rsvim_core::cli::CliOptions;
use rsvim_core::err::IoResult;
use rsvim_core::evloop::EventLoop;
use rsvim_core::evloop::mock::*;
use rsvim_core::js::JsRuntimeForSnapshot;
use rsvim_core::js::SnapshotData;
use std::path::Path;

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

fn create_event_loop(snapshot: Option<Vec<u8>>) -> EventLoop {
  let cli_opts = CliOptions::empty();
  match snapshot {
    Some(snapshot) => {
      let snapshot: &'static [u8] = Box::leak(snapshot.into_boxed_slice());
      EventLoop::mock_new_with_snapshot(
        10,
        10,
        cli_opts,
        SnapshotData::new(snapshot),
      )
      .unwrap()
    }
    None => EventLoop::mock_new_without_snapshot(10, 10, cli_opts).unwrap(),
  }
}

async fn run_event_loop(mut ev: EventLoop) -> IoResult<()> {
  ev.initialize()?;
  ev.run().await?;
  ev.shutdown()?;
  Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
  c.bench_function("startup time with snapshot", |b| {
    let tp = make_configs(vec![(
      black_box(Path::new(".rsvim.js")),
      black_box("Rsvim.rt.exit();"),
    )]);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let snapshot = create_snapshot(black_box(&tp));
    b.iter(|| {
      rt.block_on(async {
        let snapshot = snapshot.clone();
        let ev = create_event_loop(black_box(Some(snapshot)));
        let result = run_event_loop(ev).await;
        result.unwrap();
      })
    })
  });

  c.bench_function("startup time without snapshot", |b| {
    let _tp = make_configs(vec![(
      black_box(Path::new(".rsvim.js")),
      black_box("Rsvim.rt.exit();"),
    )]);
    let rt = tokio::runtime::Runtime::new().unwrap();
    b.iter(|| {
      rt.block_on(async {
        let ev = create_event_loop(black_box(None));
        let result = run_event_loop(ev).await;
        result.unwrap();
      })
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
