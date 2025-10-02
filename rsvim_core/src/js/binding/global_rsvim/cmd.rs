//! APIs for `Rsvim.cmd` namespace.

use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::binding;
use crate::js::command::def::CommandDefinition;
use crate::js::converter::*;
use crate::msg;
use crate::msg::MasterMessage;
use crate::msg::PrintReq;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::CompactString;
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
  let def = CommandDefinition::from_v8_callback_arguments(scope, args);
  trace!("Rsvim.cmd.create:{:?}", def);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);

  let result = commands
    .insert(def.name.to_compact_string(), CommandDefinition::to_rc(def));

  match result {
    Ok(Some(removed)) => rv.set(removed.to_v8(scope)),
    Ok(None) => rv.set_undefined(),
    Err(e) => {
      rv.set_undefined();
      binding::throw_exception(scope, &e);
    }
  }
}

/// `Rsvim.cmd.list` API.
pub fn list(
  scope: &mut v8::PinScope,
  _args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(_args.length() == 0);
  trace!("Rsvim.cmd.list");

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let commands = lock!(state.commands);

  let commands =
    to_v8::<Vec<CompactString>>(scope, commands.keys().cloned().collect());

  rv.set(commands);
}

/// `Rsvim.cmd.remove` API.
pub fn remove<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let name = from_v8::<CompactString>(scope, args.get(0));
  trace!("Rsvim.cmd.remove:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  match commands.remove(&name) {
    Some(removed) => rv.set(removed.to_v8(scope)),
    None => rv.set_undefined(),
  }
}
