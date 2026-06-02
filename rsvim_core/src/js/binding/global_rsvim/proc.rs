//! APIs for `Rsvim.proc` namespace.

pub mod proc_command;

use crate::is_v8_str;
use crate::js;
use crate::js::JsRuntime;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use compact_str::ToCompactString;
use proc_command::ProcCommandOptions;
use proc_command::SpawnChildProcessFuture;

/// The `spawn` method in `Rsvim.proc.Command` class.
pub fn spawn<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let exec_path = args.get(0).to_rust_string_lossy(scope);
  let options = ProcCommandOptions::from_v8(scope, args.get(1));
  trace!("spawn exec_path: {:?}, options: {:?}", exec_path, options);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let spawn_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = SpawnChildProcessFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  pending::create_spawn_child_process(
    &mut state,
    task_id,
    exec_path.to_compact_string(),
    options,
    Box::new(spawn_cb),
  );

  rv.set(promise.into());
}
