use super::js::*;
use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::evloop::writer::StdoutWriterValue;
use crate::prelude::*;
use assert_fs::prelude::PathChild;

#[test]
fn create_snapshot1() {
  // Prepare snapshot data
  let snapshot_file = {
    let js_runtime = JsRuntimeForSnapshot::new();
    let snapshot = js_runtime.create_snapshot();
    let snapshot = Box::from(&snapshot);
    let mut vec = Vec::with_capacity(snapshot.len());
    vec.extend_from_slice(&snapshot);

    let temp_dir = assert_fs::TempDir::new().unwrap();
    let output_path = temp_dir.child("snapshot.bin");
    info!("Write snapshot to {:?}", output_path.path());
    std::fs::write(output_path.path(), vec.into_boxed_slice()).unwrap();
    output_path
  };

  // Create js runtime with snapshot.
  let mut event_loop = {
    let bytes = std::fs::read(snapshot_file.path()).unwrap();

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
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      SnapshotData::new(bytes.clone().as_slice()),
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
}
