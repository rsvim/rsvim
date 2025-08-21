//! Messages between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js::JsRuntime).

// Re-export
pub use jsrt::*;
pub use master::*;

pub mod jsrt;
pub mod master;
