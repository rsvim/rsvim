//! Messages that are sent to [`JsRuntime`](crate::js::JsRuntime).

use crate::command::ExCommand;
use crate::js::JsFutureId;

use std::time::Duration;

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js::JsRuntime).
pub enum EventLoopToJsRuntimeMessage {
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
  pub command: ExCommand,
}

impl ExCommandReq {
  pub fn new(future_id: JsFutureId, command: ExCommand) -> Self {
    ExCommandReq { future_id, command }
  }
}
