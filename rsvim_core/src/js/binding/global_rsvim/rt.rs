//! APIs for `Rsvim.rt` namespace.

use crate::js::JsRuntime;
use crate::js::converter::*;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;

/// Exit editor.
pub fn exit<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let exit_code = from_v8::<i32>(scope, args.get(0)).unwrap();

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  trace!("exit: {:?}", exit_code);

  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::ExitReq(msg::ExitReq { exit_code }),
  );
}
