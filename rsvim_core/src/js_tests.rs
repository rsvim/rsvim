use super::js::*;
use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::evloop::writer::StdoutWriterValue;
use crate::prelude::*;
use crate::results::IoResult;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use assert_fs::prelude::PathChild;
use compact_str::ToCompactString;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_snapshot1() -> IoResult<()> {
  let temp_dir = assert_fs::TempDir::new().unwrap();
  let snapshot_file = temp_dir.child("snapshot.bin");

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

    let bytes = std::fs::read(snapshot_file.path()).unwrap();
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
