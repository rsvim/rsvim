//! Messages used between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::rt::JsRuntime).

// The message JsRuntime send to EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::rt::JsRuntime).
pub enum JsRuntimeToEventLoopMessage {
  Exit(Dummy),
}

// The message JsRuntime send to EventLoop }

// The message JsRuntime receive from EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::rt::JsRuntime).
pub enum EventLoopToJsRuntimeMessage {
  Exit(Dummy),
}

// The message JsRuntime receive from EventLoop }

#[derive(Debug, Default)]
/// Dummy message.
pub struct Dummy {}

impl Dummy {
  pub fn new() -> Self {
    Dummy {}
  }
}
