//! Messages that are sent to [`JsRuntime`](crate::js::JsRuntime).

use crate::js::JsFuture;
use crate::js::JsFutureId;
use crate::prelude::*;
use compact_str::CompactString;
use std::fmt::Debug;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::Instant;

#[derive(Debug)]
/// Message sent to [`JsRuntime`](crate::js::JsRuntime).
pub enum JsMessage {
  /// Event loop notify Js runtime to shutdown this thread.
  TimeoutResp(TimeoutResp),

  /// Event loop send ex command to js runtime to run.
  ExCommandReq(ExCommandReq),

  /// Master send js runtime the result of load import
  LoadImportResp(LoadImportResp),

  /// Master send js runtime the result of tick again
  TickAgainResp,
}

#[derive(Debug)]
pub struct TimeoutResp {
  pub timer_id: JsFutureId,
  pub expire_at: Instant,
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

#[derive(Debug)]
pub struct LoadImportResp {
  pub future_id: JsFutureId,
  pub source: AnyResult<String>,
}

impl LoadImportResp {
  pub fn new(future_id: JsFutureId, source: AnyResult<String>) -> Self {
    Self { future_id, source }
  }
}

/// Send js message in sync/blocking way, with tokio's "current_runtime".
pub fn sync_send_to_js(
  master_tx: Sender<JsMessage>,
  message: JsMessage,
) -> JoinHandle<()> {
  tokio::runtime::Handle::current().spawn_blocking(move || {
    master_tx.blocking_send(message).unwrap();
  })
}
