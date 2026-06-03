//! APIs for `Rsvim.proc` namespace.

pub mod proc_command;

use crate::is_v8_str;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::binding::global_rsvim::proc::proc_command::spawn_child_process;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use proc_command::ProcCommandOptions;

/// The `spawn` method in `Rsvim.proc.Command` class.
pub fn spawn_child<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_str!(args.get(0)));
  let exec_path = args.get(0).to_rust_string_lossy(scope);
  let options = ProcCommandOptions::from_v8(scope, args.get(1));
  trace!(
    "spawn_child exec_path: {:?}, options: {:?}",
    exec_path, options
  );

  let state_rc = JsRuntime::state(scope);
  let resource_table = state_rc.borrow().resource_table.clone();

  match spawn_child_process(resource_table, &exec_path, &options) {
    Ok((child_rid, stdin_rid, stdout_rid, stderr_rid)) => {
      let result = v8::Object::new(scope);
      let exec_path = exec_path.to_compact_string().to_v8(scope);
      binding::set_property_to(scope, result, "execPath", exec_path);
      let options = options.to_v8(scope);
      binding::set_property_to(scope, result, "options", options);
      let child_rid = Into::<i32>::into(child_rid).to_v8(scope);
      binding::set_property_to(scope, result, "rid", child_rid);
      if let Some(stdin_rid) = stdin_rid {
        let stdin_rid = Into::<i32>::into(stdin_rid).to_v8(scope);
        binding::set_property_to(scope, result, "stdinRid", stdin_rid);
      }
      if let Some(stdout_rid) = stdout_rid {
        let stdout_rid = Into::<i32>::into(stdout_rid).to_v8(scope);
        binding::set_property_to(scope, result, "stdoutRid", stdout_rid);
      }
      if let Some(stderr_rid) = stderr_rid {
        let stderr_rid = Into::<i32>::into(stderr_rid).to_v8(scope);
        binding::set_property_to(scope, result, "stderrRid", stderr_rid);
      }
      rv.set(result.into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
}
