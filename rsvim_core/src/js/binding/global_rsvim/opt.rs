//! APIs for `Rsvim.opt` namespace.

use crate::glovar;
use crate::js::JsRuntime;

use tracing::debug;

/// Get the _wrap_ option.
/// See: <https://vimhelp.org/options.txt.html#%27wrap%27>
/// Also known as _line-wrap_, see: <https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap>.
pub fn get_wrap(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let value = state_rc
    .borrow()
    .tree
    .try_read_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .wrap();
  debug!("get_wrap: {:?}", value);
  rv.set_bool(value);
}

/// Set the _wrap_ option.
pub fn set_wrap(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  assert!(args.length() == 1);
  let value = args.get(0).to_boolean(scope).boolean_value(scope);
  let state_rc = JsRuntime::state(scope);
  debug!("set_wrap: {:?}", value);
  state_rc
    .borrow_mut()
    .tree
    .try_write_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .set_wrap(value);
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
  let value = state_rc
    .borrow()
    .tree
    .try_read_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .line_break();
  debug!("get_line_break: {:?}", value);
  rv.set_bool(value);
}

/// Set the _line-break_ option.
pub fn set_line_break(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  assert!(args.length() == 1);
  let value = args.get(0).to_boolean(scope).boolean_value(scope);
  let state_rc = JsRuntime::state(scope);
  debug!("set_line_break: {:?}", value);
  state_rc
    .borrow_mut()
    .tree
    .try_write_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .set_line_break(value);
}

/// Get the _break-at_ option.
/// See: <https://vimhelp.org/options.txt.html#%27breakat%27>
pub fn get_break_at(
  scope: &mut v8::HandleScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  let state_rc = JsRuntime::state(scope);
  let value = state_rc
    .borrow()
    .tree
    .try_read_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .breat_at()
    .clone();
  debug!("get_break_at: {:?}", value);
  rv.set(v8::String::new(scope, &value).unwrap().into());
}

/// Set the _break-at_ option.
pub fn set_break_at(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  assert!(args.length() == 1);
  let value = args.get(0).to_rust_string_lossy(scope);
  let state_rc = JsRuntime::state(scope);
  debug!("set_break_at: {:?}", value);
  state_rc
    .borrow_mut()
    .tree
    .try_write_for(glovar::MUTEX_TIMEOUT())
    .unwrap()
    .set_break_at(&value);
}
