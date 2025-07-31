//! Messages synced between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js::JsRuntime).

use std::time::Duration;

use compact_str::CompactString;

use crate::js::JsFutureId;

// The message JsRuntime send to EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js::JsRuntime).
pub enum JsRuntimeToEventLoopMessage {
  EchoReq(EchoReq),
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

  /// Event loop send EX command to js runtime to run.
  ExCommandReq(ExCommandReq),
}

// The message JsRuntime receive from EventLoop }

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
pub struct EchoReq {
  pub message: CompactString,
}

impl EchoReq {
  pub fn new(message: CompactString) -> Self {
    EchoReq { message }
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

#[derive(Debug)]
pub struct ExCommandReq {
  pub future_id: JsFutureId,
  pub source: CompactString,
}

impl ExCommandReq {
  pub fn new(future_id: JsFutureId, source: CompactString) -> Self {
    ExCommandReq { future_id, source }
  }
}
