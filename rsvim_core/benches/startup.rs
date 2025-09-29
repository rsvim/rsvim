use compact_str::ToCompactString;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use rsvim_core::cli::CliOptions;
use rsvim_core::evloop::EventLoop;
use rsvim_core::evloop::mock::*;
use rsvim_core::evloop::writer::StdoutWriterValue;
use rsvim_core::js::*;
use rsvim_core::prelude::*;
use rsvim_core::results::IoResult;
use rsvim_core::state::ops::CursorInsertPayload;
use rsvim_core::state::ops::Operation;

fn create_snapshot(tp: &TempPathConfig) -> Vec<u8> {
  let snapshot_file = Path::new("benches_startup_snapshot.bin");

  // Prepare snapshot data
  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  info!("Write snapshot to {:?}", snapshot_file.path());
  std::fs::write(snapshot_file.path(), vec.into_boxed_slice()).unwrap();

  let bytes = std::fs::read(snapshot_file.path()).unwrap();

  bytes
}

async fn run_with_snapshot(
  tp: &TempPathConfig,
  snapshot: Vec<u8>,
) -> IoResult<()> {
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

  Ok(())
}

async fn run_without_snapshot(tp: &TempPathConfig) -> IoResult<()> {
  // Create js runtime without snapshot.
  let cli_opts = CliOptions::empty();
  let mut event_loop = make_event_loop(10, 10, cli_opts);

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

  Ok(())
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let tp = TempPathConfig::create();
  let snapshot = create_snapshot(&tp);
  let rt = tokio::runtime::Runtime::new().unwrap();

  c.bench_function("With snapshot", |b| {
    b.iter(|| {
      rt.block_on(async {
        run_with_snapshot(tp, snapshot).await;
      })
    })
  });
  c.bench_function("Without snapshot", |b| {
    b.iter(|| {
      rt.block_on(async {
        run_without_snapshot(tp).await;
      })
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
