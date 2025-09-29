use assert_fs::prelude::PathChild;
use compact_str::ToCompactString;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ringbuf::traits::*;
use rsvim_core::cli::CliOptions;
use rsvim_core::evloop::EventLoop;
use rsvim_core::evloop::writer::StdoutWriterValue;
use rsvim_core::js::*;
use rsvim_core::prelude::*;
use rsvim_core::results::IoResult;
use rsvim_core::state::ops::CursorInsertPayload;
use rsvim_core::state::ops::Operation;
use rsvim_core::tests::evloop::*;
use rsvim_core::tests::log::init as test_log_init;
use std::time::Duration;

fn create_snapshot(tp: &TempConfigDir) -> Vec<u8> {
  let snapshot_file = tp.xdg_data_home.child("snapshot.bin");

  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  info!("Write snapshot to {:?}", snapshot_file.path());
  std::fs::write(snapshot_file.path(), vec.into_boxed_slice()).unwrap();

  let snapshot = std::fs::read(snapshot_file.path()).unwrap();
  snapshot
}

async fn with_snapshot(tp: &TempConfigDir, snapshot: Vec<u8>) -> IoResult<()> {
  // Create js runtime with snapshot.
  let mut event_loop = {
    let cli_opts = CliOptions::empty();
    let (
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
      (jsrt_tx, jsrt_rx),
    ) = EventLoop::_internal_new(10, 10).unwrap();

    let writer = StdoutWriterValue::dev_null();
    let bytes: &'static [u8] = Box::leak(snapshot.into_boxed_slice());

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      SnapshotData::new(bytes),
      startup_moment,
      startup_unix_epoch,
      master_tx.clone(),
      jsrt_rx,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
      commands,
    );

    EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      js_runtime,
      master_tx,
      master_rx,
      jsrt_forwarder_tx,
      jsrt_forwarder_rx,
      jsrt_tx,
    }
  };

  // Run the event loop.
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("js Rsvim.cmd.echo(1);".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
  ];

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}

async fn with_snapshot() -> IoResult<()> {
  test_log_init();

  // Prepare $RSVIM_CONFIG/rsvim.js
  let tp = make_configs(vec![(Path::new("rsvim.js"), "")]);

  let snapshot_file = tp.xdg_data_home.child("snapshot.bin");

  // Prepare snapshot data
  {
    let js_runtime = JsRuntimeForSnapshot::new();
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend_from_slice(&snapshot);

    info!("Write snapshot to {:?}", snapshot_file.path());
    std::fs::write(snapshot_file.path(), vec.into_boxed_slice()).unwrap();
  };

  let bytes = std::fs::read(snapshot_file.path()).unwrap();

  // Create js runtime with snapshot.
  let mut event_loop = {
    let cli_opts = CliOptions::empty();
    let (
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
      (jsrt_tx, jsrt_rx),
    ) = EventLoop::_internal_new(10, 10).unwrap();

    let writer = StdoutWriterValue::dev_null();

    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      SnapshotData::new(bytes),
      startup_moment,
      startup_unix_epoch,
      master_tx.clone(),
      jsrt_rx,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
      commands,
    );

    EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      js_runtime,
      master_tx,
      master_rx,
      jsrt_forwarder_tx,
      jsrt_forwarder_rx,
      jsrt_tx,
    }
  };

  // Run the event loop.
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("js Rsvim.cmd.echo(1);".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
    MockOperation::SleepFor(Duration::from_millis(50)),
  ];

  event_loop.initialize()?;
  event_loop
    .run_with_mock_operations(MockOperationReader::new(mocked_ops))
    .await?;
  event_loop.shutdown()?;

  // After running
  {
    let mut contents = lock!(event_loop.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    info!("actual:{:?}", actual);
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let tp = make_configs(vec![(Path::new("rsvim.js"), "")]);

  c.bench_function("with snapshot", |b| b.iter(|| fibonacci(black_box(20))));
  c.bench_function("without snapshot", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
