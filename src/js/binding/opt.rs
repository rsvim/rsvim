//! APIs for `__vim.opt` namespace.

use crate::glovar;
use crate::js::JsRuntime;

use std::time::Duration;

/// Get line wrap option.
pub fn line_wrap(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let value = state_rc
    .borrow()
    .tree
    .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
    .unwrap()
    .line_wrap();
  rv.set_bool(true);
}

/// Set line wrap option.
pub fn set_line_wrap(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  assert!(args.length() == 1);
  let value = args.get(0).to_boolean(scope).boolean_value(scope);
  let state_rc = JsRuntime::state(scope);
  state_rc
    .borrow()
    .tree
    .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
    .unwrap()
    .set_line_wrap(value);
}
