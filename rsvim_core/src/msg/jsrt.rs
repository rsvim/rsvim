//! Messages that are sent to [`JsRuntime`](crate::js::JsRuntime).

use crate::js::JsTaskId;
use crate::js::JsTimerId;
use crate::prelude::*;
use compact_str::CompactString;
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
  pub timer_id: JsTimerId,
  pub expire_at: Instant,
  pub delay: u64,
  pub repeated: bool,
}

#[derive(Debug)]
pub struct ExCommandReq {
  pub payload: CompactString,
}

#[derive(Debug)]
pub struct LoadImportResp {
  pub task_id: JsTaskId,
  pub maybe_source: Option<TheResult<Vec<u8>>>,
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
