use super::js::*;
use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;
use crate::state::ops::CursorInsertPayload;
use crate::state::ops::Operation;
use crate::tests::evloop::*;
use crate::tests::log::init as test_log_init;
use assert_fs::prelude::PathChild;
use compact_str::ToCompactString;
use ringbuf::traits::*;
use std::time::Duration;

#[tokio::test]
#[cfg_attr(miri, ignore)]
async fn create_snapshot1() -> IoResult<()> {
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
    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    EventLoop::mock_new_with_snapshot(
      10,
      10,
      cli_opts,
      SnapshotData::new(bytes),
    )
    .unwrap()
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
