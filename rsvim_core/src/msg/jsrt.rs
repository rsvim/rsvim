//! Messages that are sent to [`JsRuntime`](crate::js::JsRuntime).

use crate::js::JsFutureId;

use compact_str::CompactString;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

#[derive(Debug)]
/// Message sent to [`JsRuntime`](crate::js::JsRuntime).
pub enum JsMessage {
  /// Event loop notify Js runtime to shutdown this thread.
  TimeoutResp(TimeoutResp),

  /// Event loop send ex command to js runtime to run.
  ExCommandReq(ExCommandReq),
}

#[derive(Debug)]
pub struct TimeoutResp {
  pub future_id: JsFutureId,
  pub duration: Duration,
}

impl TimeoutResp {
  pub fn new(future_id: JsFutureId, duration: Duration) -> Self {
    TimeoutResp {
      future_id,
      duration,
    }
  }
}

#[derive(Debug)]
pub struct ExCommandReq {
  pub future_id: JsFutureId,
  pub payload: CompactString,
}

impl ExCommandReq {
  pub fn new(future_id: JsFutureId, payload: CompactString) -> Self {
    ExCommandReq { future_id, payload }
  }
}

/// Send js message in sync/blocking way, with tokio's "current_runtime".
pub fn sync_send_js(
  master_tx: Sender<JsMessage>,
  message: JsMessage,
) -> JoinHandle<()> {
  tokio::runtime::Handle::current().spawn_blocking(move || {
    master_tx.blocking_send(message).unwrap();
  })
}
