//! APIs for `Rsvim.opt` namespace.

use crate::js::JsRuntime;
use crate::prelude::*;

/// Get the _wrap_ option.
/// See: <https://vimhelp.org/options.txt.html#%27wrap%27>
/// Also known as _line-wrap_, see: <https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap>.
pub fn get_wrap(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  let value = tree.global_local_options().wrap();
  trace!("get_wrap: {:?}", value);
  rv.set_bool(value);
}

/// Set the _wrap_ option.
pub fn set_wrap(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).boolean_value(scope);
  trace!("set_wrap: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let mut tree = lock!(tree);
  tree.global_local_options_mut().set_wrap(value);
}

/// Get the _line-break_ option.
/// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>
/// Also known as _word-wrap_, see: <https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap>.
pub fn get_line_break(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  let value = tree.global_local_options().line_break();
  trace!("get_line_break: {:?}", value);
  rv.set_bool(value);
}

/// Set the _line-break_ option.
pub fn set_line_break(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).boolean_value(scope);
  trace!("set_line_break: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let mut tree = lock!(tree);
  tree.global_local_options_mut().set_line_break(value);
}

/// Get the _tap-stop_ option.
/// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>
pub fn get_tab_stop(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  let value = buffers.global_local_options().tab_stop();
  trace!("get_tab_stop: {:?}", value);
  rv.set_int32(value as i32);
}

/// Set the _tab-stop_ option.
pub fn set_tab_stop(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).int32_value(scope).unwrap();
  trace!("set_tab_stop: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let mut buffers = lock!(buffers);

  let value = num_traits::clamp(value, 0, u16::MAX as i32) as u16;
  buffers.global_local_options_mut().set_tab_stop(value);
}
