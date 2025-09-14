//! Messages that are sent to [`EventLoop`](crate::evloop::EventLoop), here
//! call it "master".

use crate::js::JsFutureId;
use crate::js::JsTaskId;
use compact_str::CompactString;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio::time::Instant;

#[derive(Debug)]
/// Message sent to [`EventLoop`](crate::evloop::EventLoop).
pub enum MasterMessage {
  /// Js runtime ask master to print message
  PrintReq(PrintReq),

  /// Js runtime ask master to set timeout, i.e. sleep
  TimeoutReq(TimeoutReq),

  /// Js runtime ask master to exit
  ExitReq(ExitReq),

  /// Js runtime ask master to load import
  LoadImportReq(LoadImportReq),

  /// Js runtime ask master to tick loop again.
  TickAgainReq,
}

#[derive(Debug)]
pub struct ExitReq {
  pub exit_code: i32,
}

#[derive(Debug)]
pub struct PrintReq {
  pub payload: CompactString,
}

#[derive(Debug)]
pub struct TimeoutReq {
  pub timer_id: JsFutureId,
  pub expire_at: Instant,
}

#[derive(Debug)]
pub struct LoadImportReq {
  pub task_id: JsTaskId,
  pub specifier: String,
}

/// Send master message in sync/blocking way, with tokio's "current_runtime".
pub fn sync_send_to_master(
  master_tx: Sender<MasterMessage>,
  message: MasterMessage,
) -> JoinHandle<()> {
  tokio::runtime::Handle::current().spawn_blocking(move || {
    master_tx.blocking_send(message).unwrap();
  })
}
