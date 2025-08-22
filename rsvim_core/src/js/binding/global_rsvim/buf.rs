use crate::js::{self, JsRuntime};
use crate::msg::{self, MasterMessage};
use crate::prelude::*;

use compact_str::ToCompactString;

/// `Rsvim.buf.write` API.
pub fn write(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let buf_id = args.get(0).int32_value(scope).unwrap();
  trace!("write: {:?}", buf_id);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  let buffers = state.buffers.clone();
  let buffers = lock!(buffers);

  match buffers.get(&buf_id) {
    Some(buf) => {
      let mut buf = lock!(buf);
      buf.write();
      msg::sync_send_to_master(
        state.master_tx.clone(),
        MasterMessage::PrintReq(msg::PrintReq::new(
          js::next_future_id(),
          message.to_compact_string(),
        )),
      );
    }
    None => {
      msg::sync_send_to_master(
        state.master_tx.clone(),
        MasterMessage::PrintReq(msg::PrintReq::new(
          message_id,
          message.to_compact_string(),
        )),
      );
    }
  }
}
