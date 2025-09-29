use assert_fs::prelude::PathChild;
use compact_str::ToCompactString;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
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

fn create_snapshot(tp: &TempConfigDir) -> Vec<u8> {
  let snapshot_file = tp.xdg_data_home.child("snapshot.bin");

  let js_runtime = JsRuntimeForSnapshot::new();
  let snapshot = js_runtime.create_snapshot();
  let snapshot = Box::from(&snapshot);
  let mut vec = Vec::with_capacity(snapshot.len());
  vec.extend_from_slice(&snapshot);

  info!("Write snapshot to {:?}", snapshot_file.path());
  std::fs::write(snapshot_file.path(), vec.into_boxed_slice()).unwrap();

  std::fs::read(snapshot_file.path()).unwrap()
}

fn create_event_loop(snapshot: Option<Vec<u8>>) -> EventLoop {
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

  // Js Runtime
  let js_runtime = match snapshot {
    Some(snapshot) => {
      let bytes: &'static [u8] = Box::leak(snapshot.into_boxed_slice());
      JsRuntime::new(
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
      )
    }
    None => JsRuntime::new_without_snapshot(
      JsRuntimeOptions::default(),
      startup_moment,
      startup_unix_epoch,
      master_tx.clone(),
      jsrt_rx,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
      commands,
    ),
  };

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
}

async fn run_event_loop(mut ev: EventLoop) -> IoResult<()> {
  // Run the event loop.
  let mocked_ops = vec![
    MockOperation::Operation(Operation::GotoCommandLineExMode),
    MockOperation::Operation(Operation::CursorInsert(
      CursorInsertPayload::Text("js Rsvim.cmd.echo(1);".to_compact_string()),
    )),
    MockOperation::Operation(Operation::ConfirmExCommandAndGotoNormalMode),
  ];

  ev.initialize()?;
  ev._run_with_mocked_operations(MockOperationReader::new(mocked_ops))
    .await?;
  ev.shutdown()?;

  // After running
  {
    let mut contents = lock!(ev.contents);
    let n = contents.command_line_message_history().occupied_len();
    assert_eq!(n, 1);

    let actual = contents.command_line_message_history_mut().try_pop();
    assert!(actual.is_some());
    let actual = actual.unwrap();
    assert_eq!(actual, "1");
  }

  Ok(())
}

async fn with_snapshot(snapshot: Vec<u8>) -> IoResult<()> {
  // Create js runtime with snapshot.
  let event_loop = create_event_loop(Some(snapshot));

  run_event_loop(event_loop).await
}

async fn without_snapshot() -> IoResult<()> {
  let event_loop = create_event_loop(None);

  run_event_loop(event_loop).await
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let tp = make_configs(vec![(Path::new("rsvim.js"), "")]);

  c.bench_function("with snapshot", |b| {
    let snapshot = create_snapshot(&tp);
    let rt = tokio::runtime::Runtime::new().unwrap();

    b.iter(|| {
      let snapshot = snapshot.clone();
      rt.block_on(async move {
        let result = with_snapshot(snapshot).await;
        result.unwrap();
      });
    })
  });
  c.bench_function("without snapshot", |b| {
    let rt = tokio::runtime::Runtime::new().unwrap();

    b.iter(|| {
      rt.block_on(async {
        let result = without_snapshot().await;
        result.unwrap();
      });
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
