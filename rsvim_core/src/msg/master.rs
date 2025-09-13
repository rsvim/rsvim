//! Messages that are sent to [`EventLoop`](crate::evloop::EventLoop), here
//! call it "master".

use crate::js::JsFuture;
use crate::js::JsFutureId;
use compact_str::CompactString;
use std::fmt::Debug;
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
  pub future_id: JsFutureId,
  pub exit_code: i32,
}

impl ExitReq {
  pub fn new(future_id: JsFutureId, exit_code: i32) -> Self {
    Self {
      future_id,
      exit_code,
    }
  }
}

#[derive(Debug)]
pub struct PrintReq {
  pub future_id: JsFutureId,
  pub payload: CompactString,
}

impl PrintReq {
  pub fn new(future_id: JsFutureId, payload: CompactString) -> Self {
    PrintReq { future_id, payload }
  }
}

pub struct TimeoutReq {
  pub timer_id: JsFutureId,
  pub expire_at: Instant,
  pub cb: Box<dyn FnMut() -> Box<dyn JsFuture> + 'static>,
}

impl Debug for TimeoutReq {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TimeoutReq")
      .field("timer_id", &self.timer_id)
      .field("expire_at", &self.expire_at)
      .field("cb", &"Box<dyn FnMut() -> Box<dyn JsFuture> + 'static>")
      .finish()
  }
}

#[derive(Debug)]
pub struct LoadImportReq {
  pub future_id: JsFutureId,
  pub specifier: String,
}

impl LoadImportReq {
  pub fn new(future_id: JsFutureId, specifier: String) -> Self {
    Self {
      future_id,
      specifier,
    }
  }
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
