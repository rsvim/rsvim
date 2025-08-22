use crate::js::{self, JsRuntime};
use crate::msg::{self, MasterMessage};
use crate::prelude::*;

use compact_str::ToCompactString;

/// `Rsvim.buf.currentBufferId` API.
pub fn current_buffer_id(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  let value = tree.global_local_options().wrap();
  trace!("get_wrap: {:?}", value);
  rv.set_int32(value);
}

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

  let message_id = js::next_future_id();
  match buffers.write_buffer(buf_id) {
    Ok(n) => {
      let buf = buffers.get(&buf_id).unwrap();
      let buf = lock!(buf);
      let message = format!(
        "Buffer {:?} ({buf_id}) has been saved ({n} bytes written).",
        buf
          .filename()
          .as_ref()
          .map(|p| String::from(p.as_path().to_string_lossy()))
          .unwrap_or("<unknown>".to_string())
          .to_owned()
      );
      msg::sync_send_to_master(
        state.master_tx.clone(),
        MasterMessage::PrintReq(msg::PrintReq::new(
          message_id,
          message.to_compact_string(),
        )),
      );
    }
    Err(e) => {
      msg::sync_send_to_master(
        state.master_tx.clone(),
        MasterMessage::PrintReq(msg::PrintReq::new(
          message_id,
          e.to_compact_string(),
        )),
      );
    }
  }
}
