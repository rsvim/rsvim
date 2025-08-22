use std::arch::aarch64::veor3q_s16;

use crate::buf::BufferId;
use crate::js::{self, JsRuntime};
use crate::msg::{self, MasterMessage};
use crate::prelude::*;

use compact_str::ToCompactString;

/// `Rsvim.buf.currentBuffer` API.
pub fn current_buffer(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  let current_window = tree.current_window().unwrap();
  let buf = current_window.buffer().upgrade().unwrap();
  let buf_id = lock!(buf).id();
  trace!("current_buffer: {:?}", buf_id);
  rv.set_int32(buf_id);
}

/// `Rsvim.buf.listAllBuffers` API.
pub fn list_all_buffers(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  trace!("list_all_buffers: {:?}", buffers.keys());
  let buf_ids = buffers.keys().copied().collect::<Vec<BufferId>>();

  let buf_ids_array = v8::Array::new(scope, buf_ids.len() as i32);
  for (i, buf_id) in buf_ids.iter().enumerate() {
    let v = v8::Integer::new(scope, *buf_id);
    let v = v8::Local::new(scope, v);
    buf_ids_array.set_index(scope, i as u32, v.into());
  }
  rv.set(v8::Local::new(scope, buf_ids_array).into());
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
