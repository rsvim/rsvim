//! Messages between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js::JsRuntime).

// Re-export
pub use jsrt_sink::*;
pub use master_sink::*;

pub mod jsrt_sink;
pub mod master_sink;
