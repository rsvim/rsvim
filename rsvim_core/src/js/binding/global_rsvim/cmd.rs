use crate::js::{self, JsRuntime};
use crate::msg::{self, JsRuntimeToEventLoopMessage};
use crate::prelude::*;

use compact_str::CompactString;

/// Javascript `echo` API.
pub fn echo(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope).to_string();
  trace!("echo: {:?}", message);

  let message_id = js::next_future_id();

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let jsrt_to_master = state.jsrt_to_master.clone();
  let current_handle = tokio::runtime::Handle::current();
  current_handle.spawn_blocking(move || {
    let message = CompactString::from(message);
    jsrt_to_master
      .blocking_send(JsRuntimeToEventLoopMessage::PrintReq(msg::PrintReq::new(
        message_id, message,
      )))
      .unwrap();
  });
}
