//! APIs for `Rsvim.cmd` namespace.

use crate::js::{JsRuntime, JsRuntimeState};
use crate::msg;
use crate::msg::MasterMessage;
use crate::msg::PrintReq;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::{CompactString, ToCompactString};

pub fn send_cmdline_message(state: &JsRuntimeState, payload: CompactString) {
  trace!("|cmd| send_cmdline_message:{:?}", payload);
  let mut tree = lock!(state.tree);
  let mut contents = lock!(state.contents);
  if tree.command_line_id().is_some() {
    cmdline_ops::cmdline_set_message(&mut tree, &mut contents, payload);
  } else {
    msg::sync_send_to_master(
      state.master_tx.clone(),
      MasterMessage::PrintReq(PrintReq { payload }),
    );
  }
}

/// `Rsvim.cmd.echo` API.
pub fn echo(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope);
  // trace!("echo: {:?}", message);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  send_cmdline_message(&state, message.to_compact_string());
}
