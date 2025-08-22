use crate::js::{self, JsRuntime};
use crate::msg::{self, MasterMessage};
use crate::prelude::*;

use compact_str::ToCompactString;

/// `Rsvim.cmd.echo` API.
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
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::PrintReq(msg::PrintReq::new(
      message_id,
      message.to_compact_string(),
    )),
  );
}
