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
  PrintReq(PrintReq),
  TimeoutReq(TimeoutReq),
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
    TimeoutReq {
      future_id,
      duration,
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
