//! APIs for `Rsvim.cmd` namespace.

use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::command::attr::CommandAttributes;
use crate::js::command::opt::CommandOptions;
use crate::msg;
use crate::msg::MasterMessage;
use crate::msg::PrintReq;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::ToCompactString;
use std::rc::Rc;

pub fn send_cmdline_message(state: &JsRuntimeState, payload: String) {
  trace!("|cmd| send_cmdline_message:{:?}", payload);
  let mut tree = lock!(state.tree);
  let mut contents = lock!(state.contents);
  if tree.command_line_id().is_some() {
    cmdline_ops::cmdline_set_message(&mut tree, &mut contents, payload);
  } else {
    msg::sync_send_to_master(
      state.master_tx.clone(),
      MasterMessage::PrintReq(PrintReq { payload }),
    );
  }
}

/// `Rsvim.cmd.echo` API.
pub fn echo(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.echo:{:?}", message);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
  send_cmdline_message(&state, message);
}

/// `Rsvim.cmd.create` API.
pub fn create(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 4);
  let name = args.get(0).to_rust_string_lossy(scope);
  let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
  let callback = Rc::new(v8::Global::new(scope, callback));
  let attrs = args.get(2).to_object(scope).unwrap();
  let attrs = CommandAttributes::from_object(scope, attrs);
  let opts = args.get(3).to_object(scope).unwrap();
  let opts = CommandOptions::from_object(scope, opts);
  trace!("Rsvim.cmd.create:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  commands.insert(name.to_compact_string(), (callback, attrs, opts));
}

/// `Rsvim.cmd.list` API.
pub fn list(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 0);
  trace!("Rsvim.cmd.list");

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let commands = lock!(state.commands);

  let cmds = v8::Array::new(scope, commands.len() as i32);
  for (i, (name, def)) in commands.iter().enumerate() {
    let cmd = v8::Object::new(scope);

    // name
    let name_field = v8::String::new(scope, "name").unwrap();
    let name_value = v8::String::new(scope, name.as_ref()).unwrap();
    cmd.set(scope, name_field.into(), name_value.into());

    // attribute
    let attr = v8::Object::new(scope);

    cmds.set_index(scope, i as u32, cmd.into());
  }

  rv.set(v8::Local::new(scope, cmds).into());
}

/// `Rsvim.cmd.remove` API.
pub fn remove(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 4);
  let name = args.get(0).to_rust_string_lossy(scope);
  let callback = v8::Local::<v8::Function>::try_from(args.get(1)).unwrap();
  let callback = Rc::new(v8::Global::new(scope, callback));
  let attrs = args.get(2).to_object(scope).unwrap();
  let attrs = CommandAttributes::from_object(scope, attrs);
  let opts = args.get(3).to_object(scope).unwrap();
  let opts = CommandOptions::from_object(scope, opts);
  trace!("Rsvim.cmd.create:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  commands.insert(name.to_compact_string(), (callback, attrs, opts));
}
