//! APIs for `Rsvim.opt` namespace.

use crate::buf::opt::FileEncodingOption;
use crate::buf::opt::FileFormatOption;
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

  let value = num_traits::clamp(value, 0, u8::MAX as i32) as u16;
  buffers.global_local_options_mut().set_tab_stop(value);
}

/// Get the _expand-tab_ option.
/// See: <https://vimhelp.org/options.txt.html#%27expandtab%27>
pub fn get_expand_tab(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  let value = buffers.global_local_options().expand_tab();
  trace!("get_expand_tab: {:?}", value);
  rv.set_bool(value);
}

/// Set the _expand-tab_ option.
pub fn set_expand_tab(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).boolean_value(scope);
  trace!("set_expand_tab: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let mut buffers = lock!(buffers);

  buffers.global_local_options_mut().set_expand_tab(value);
}

/// Get the _shift-width_ option.
/// See: <https://vimhelp.org/options.txt.html#%27shiftwidth%27>
pub fn get_shift_width(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  let value = buffers.global_local_options().shift_width();
  trace!("get_shift_width: {:?}", value);
  rv.set_int32(value as i32);
}

/// Set the _shift-width_ option.
pub fn set_shift_width(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).int32_value(scope).unwrap();
  trace!("set_shift_width: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let mut buffers = lock!(buffers);

  let value = num_traits::clamp(value, 0, u8::MAX as i32) as u16;
  buffers.global_local_options_mut().set_shift_width(value);
}

/// Get the _file-encoding_ option.
/// See: <https://vimhelp.org/options.txt.html#%27fileencoding%27>
pub fn get_file_encoding(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  let value = buffers.global_local_options().file_encoding();
  trace!("get_file_encoding: {:?}", value);
  let value = v8::String::new(scope, &value.to_string()).unwrap();
  rv.set(value.into());
}

/// Set the _file-encoding_ option.
pub fn set_file_encoding(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).to_rust_string_lossy(scope).to_lowercase();
  trace!("set_file_encoding: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let mut buffers = lock!(buffers);

  let value = FileEncodingOption::try_from(value.as_str()).unwrap();
  buffers.global_local_options_mut().set_file_encoding(value);
}

/// Get the _file-format_ option.
/// See: <https://vimhelp.org/options.txt.html#%27fileformat%27>
pub fn get_file_format(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let buffers = lock!(buffers);
  let value = buffers.global_local_options().file_format();
  trace!("get_file_format: {:?}", value);
  let value = v8::String::new(scope, &value.to_string()).unwrap();
  rv.set(value.into());
}

/// Set the _file-format_ option.
pub fn set_file_format(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let value = args.get(0).to_rust_string_lossy(scope).to_lowercase();
  trace!("set_file_format: {:?}", value);
  let state_rc = JsRuntime::state(scope);
  let buffers = state_rc.borrow().buffers.clone();
  let mut buffers = lock!(buffers);

  let value = FileFormatOption::try_from(value.as_str()).unwrap();
  buffers.global_local_options_mut().set_file_format(value);
}
