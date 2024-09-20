//! Messages used between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js_runtime::JsRuntime).

// The message JsRuntime send to EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js_runtime::JsRuntime).
pub enum JsRuntimeToEventLoopMessage {
  Dummy(Dummy),
}

// The message JsRuntime send to EventLoop }

// The message JsRuntime receive from EventLoop {

#[derive(Debug)]
/// Message between [`EventLoop`](crate::evloop::EventLoop) and
/// [`JsRuntime`](crate::js_runtime::JsRuntime).
pub enum EventLoopToJsRuntimeMessage {
  /// Event loop notify Js runtime to shutdown this thread.
  Shutdown(Dummy),
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
