//! APIs for `Rsvim.cmd` namespace.

use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::command::def::CommandDefinition;
use crate::js::converter::*;
use crate::msg;
use crate::msg::MasterMessage;
use crate::msg::PrintReq;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::ToCompactString;

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
  scope: &mut v8::PinScope,
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
pub fn create<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  let def = CommandDefinition::from_v8_callback_arguments(scope, args).unwrap();
  trace!("Rsvim.cmd.create:{:?}", def);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  let removed = commands
    .insert(def.name.to_compact_string(), CommandDefinition::to_rc(def));

  match removed {
    Ok(Some(removed)) => rv.set(removed.to_v8(scope).unwrap()),
    Ok(None) => rv.set_undefined(),
    Err(e) => {}
  }
}

/// `Rsvim.cmd.list` API.
pub fn list(
  scope: &mut v8::PinScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 0);
  trace!("Rsvim.cmd.list");

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let commands = lock!(state.commands);

  let cmds = v8::Array::new(scope, commands.len() as i32);

  for (i, def) in commands.values().enumerate() {
    let v = def.to_v8(scope);
    cmds.set_index(scope, i as u32, v.into());
  }

  rv.set(v8::Local::new(scope, cmds).into());
}

/// `Rsvim.cmd.remove` API.
pub fn remove(
  scope: &mut v8::PinScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let name = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.remove:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  match commands.remove(&name) {
    Some(removed) => {
      let obj = removed.to_v8(scope);
      rv.set(obj.into());
    }
    None => rv.set_undefined(),
  }
}
