//! Messages that are sent to [`EventLoop`](crate::evloop::EventLoop), here
//! call it "master".

use crate::js::JsFutureId;

use compact_str::CompactString;
use std::time::Duration;

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js::JsRuntime).
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
