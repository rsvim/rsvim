//! APIs for `Rsvim.rt` namespace.

use crate::is_v8_int;
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
  debug_assert!(is_v8_int!(args.get(0)));
  let exit_code = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  trace!("exit: {:?}", exit_code);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();

  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::ExitReq(msg::ExitReq { exit_code }),
  );
}
