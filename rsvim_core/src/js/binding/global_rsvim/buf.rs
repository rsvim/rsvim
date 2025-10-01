//! APIs for `Rsvim.buf` namespace.

use crate::buf::BufferId;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;

/// `Rsvim.buf.current` API.
pub fn current(
  scope: &mut v8::PinScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  match tree.current_window() {
    Some(current_window) => {
      let buf = current_window.buffer().upgrade().unwrap();
      let buf_id = lock!(buf).id();
      trace!("Rsvim.buf.current: {:?}", buf_id);
      rv.set_int32(buf_id);
    }
    None => {
      trace!("Rsvim.buf.current: not exist");
      rv.set_undefined();
    }
  }
}

/// `Rsvim.buf.list` API.
pub fn list(
  scope: &mut v8::PinScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  trace!("Rsvim.buf.list: {:?}", buffers.keys());

  let bufs =
    to_v8::<Vec<BufferId>>(scope, buffers.keys().copied().collect()).unwrap();

  rv.set(bufs);
}

/// `Rsvim.buf.writeSync` API.
pub fn write_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let buf_id = from_v8::<BufferId>(scope, args.get(0)).unwrap();
  trace!("Rsvim.buf.writeSync: {:?}", buf_id);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  let buffers = state.buffers.clone();
  let buffers = lock!(buffers);

  match buffers.write_buffer(buf_id) {
    Ok(n) => {
      trace!("write_sync success, bufId:{:?}, bytes:{:?}", buf_id, n);
      rv.set_int32(n as i32);
    }
    Err(e) => {
      trace!("write_sync failed, bufId:{:?}, error:{:?}", buf_id, e);
      rv.set_undefined();
      binding::throw_exception(scope, &e);
    }
  }
}
