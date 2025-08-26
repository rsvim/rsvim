//! APIs for `Rsvim.rt` namespace.

use crate::js::JsRuntime;
use crate::prelude::*;

/// Get the _wrap_ option.
/// See: <https://vimhelp.org/options.txt.html#%27wrap%27>
/// Also known as _line-wrap_, see: <https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap>.
pub fn exit(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let exit_code = args.get(0).int32_value(scope);

  let state_rc = JsRuntime::state(scope);
  let tree = state_rc.borrow().tree.clone();
  let tree = lock!(tree);
  let value = tree.global_local_options().wrap();
  trace!("get_wrap: {:?}", value);
  rv.set_bool(value);
}
