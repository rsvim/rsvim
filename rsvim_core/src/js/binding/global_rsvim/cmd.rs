//! APIs for `Rsvim.cmd` namespace.

use crate::js::{self, JsRuntime};
use crate::msg::{self, MasterMessage};
use crate::prelude::*;
use crate::state::ops::cmdline_ops;

use compact_str::ToCompactString;

/// `Rsvim.cmd.echo` API.
pub fn echo(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope);
  trace!("echo: {:?}", message);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();

  let mut tree = lock!(state.tree);
  let mut contents = lock!(state.contents);
  if tree.command_line_id().is_some() {
    cmdline_ops::cmdline_set_message(
      &mut tree,
      &mut contents,
      message.to_compact_string(),
    );
  } else {
    msg::sync_send_to_master(
      state.master_tx.clone(),
      MasterMessage::PrintReq(msg::PrintReq::new(
        js::next_future_id(),
        message.to_compact_string(),
      )),
    );
  }
}
