//! APIs for `Rsvim.cmd` namespace.

use crate::is_v8_nil;
use crate::is_v8_str;
use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::binding;
use crate::js::command::def::CommandDefinition;
use crate::js::converter::*;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::CompactString;
use compact_str::ToCompactString;
use ringbuf::traits::RingBuffer;

pub fn send_cmdline_message(state: &JsRuntimeState, payload: String) {
  trace!("|cmd| send_cmdline_message:{:?}", payload);
  let mut tree = lock!(state.tree);
  let mut contents = lock!(state.contents);
  if tree.command_line_id().is_some() {
    cmdline_ops::cmdline_set_message(&mut tree, &mut contents, payload);
  } else {
    // If `command_line` widget does not exist, it means the TUI is not
    // initialized yet. All we can do is simply store this message to
    // command-line-message-history. When editor TUI initialize, it will flush
    // all pending messages to TUI before running the event loop.
    //
    // See [crate::evloop::EventLoop::flush_pending_command_line_messages].
    contents
      .command_line_message_history_mut()
      .push_overwrite(payload);
  }
}

/// `Rsvim.cmd.echo` API.
pub fn echo(
  scope: &mut v8::PinScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(!is_v8_nil!(args.get(0)));
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
    Ok(Some(removed)) => rv.set(removed.to_v8(scope).into()),
    Ok(None) => rv.set_undefined(),
    Err(e) => {
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

  let commands = commands
    .keys()
    .collect::<Vec<&CompactString>>()
    .to_v8(scope, |scope, cmd| cmd.to_v8(scope).into());

  rv.set(commands.into());
}

/// `Rsvim.cmd.get` API.
pub fn get<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let name = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.get:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let commands = lock!(state.commands);
  match commands.get(&name) {
    Some(def) => rv.set(def.to_v8(scope).into()),
    None => rv.set_undefined(),
  }
}

/// `Rsvim.cmd.remove` API.
pub fn remove<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let name = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.remove:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  match commands.remove(&name) {
    Some(removed) => rv.set(removed.to_v8(scope).into()),
    None => rv.set_undefined(),
  }
}
