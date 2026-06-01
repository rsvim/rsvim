//! APIs for `Rsvim.proc` namespace.

pub mod proc_command;

use crate::chan;
use crate::chan::MasterMessage;
use crate::is_v8_str;
use crate::js::JsRuntime;
use crate::js::converter::*;
use crate::prelude::*;

/// The `spawn` method in `Rsvim.proc.Command` class.
pub fn spawn<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let exec_path = args.get(0).to_rust_string_lossy(scope);
  trace!("spawn exec_path: {:?}", exec_path);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();

  chan::send_to_master(
    state.master_tx.clone(),
    MasterMessage::ExitReq(chan::ExitReq { exit_code }),
  );
}
