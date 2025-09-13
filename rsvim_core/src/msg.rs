//! Messages between [`EventLoop`](crate::evloop::EventLoop) and
//! [`JsRuntime`](crate::js::JsRuntime).

pub mod jsrt;
pub mod master;

pub use jsrt::*;
pub use master::*;
