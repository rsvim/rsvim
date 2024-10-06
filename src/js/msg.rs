//! Messages synced between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js::JsRuntime).

use std::time::Duration;

use crate::js::JsFutureId;

// The message JsRuntime send to EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js::JsRuntime).
pub enum JsRuntimeToEventLoopMessage {
  TimeoutReq(TimeoutReq),
}

// The message JsRuntime send to EventLoop }

// The message JsRuntime receive from EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js::JsRuntime).
pub enum EventLoopToJsRuntimeMessage {
  /// Event loop notify Js runtime to shutdown this thread.
  TimeoutResp(TimeoutResp),
}

// The message JsRuntime receive from EventLoop }

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
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
