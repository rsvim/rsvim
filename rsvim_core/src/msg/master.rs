//! Messages that are sent to [`EventLoop`](crate::evloop::EventLoop), here
//! call it "master".

use crate::js::JsFutureId;

use compact_str::CompactString;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

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

#[derive(Debug)]
pub struct TimeoutReq {
  pub future_id: JsFutureId,
  pub duration: Duration,
}

impl TimeoutReq {
  pub fn new(future_id: JsFutureId, duration: Duration) -> Self {
    Self {
      future_id,
      duration,
    }
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
