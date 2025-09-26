//! APIs for `Rsvim.cmd` namespace.

use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::command::attr::CommandAttributes;
use crate::js::command::def::CommandDefinition;
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

fn command_def_to_v8_obj<'a>(
  scope: &mut v8::HandleScope<'a>,
  cmd_def: CommandDefinition,
) -> v8::Local<'a, v8::Object> {
  let cmd = v8::Object::new(scope);

  // name
  let name_field = v8::String::new(scope, "name").unwrap();
  let name_value = v8::String::new(scope, name.as_ref()).unwrap();
  cmd.set(scope, name_field.into(), name_value.into());

  // attributes
  let attr_field = v8::String::new(scope, "attributes").unwrap();
  let attr_value = def.attributes.into_v8_object(scope);
  cmd.set(scope, attr_field.into(), attr_value.into());

  // options
  let opts_field = v8::String::new(scope, "options").unwrap();
  let opts_value = def.options.into_v8_object(scope);
  cmd.set(scope, opts_field.into(), opts_value.into());

  cmd
}

/// `Rsvim.cmd.create` API.
pub fn create(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 4);
  let name = args.get(0).to_rust_string_lossy(scope);
  let def = CommandDefinition::from_v8_object(scope, args);
  trace!("Rsvim.cmd.create, name:{:?}, def:{:?}", name, def);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  let removed = commands.insert(
    name.to_compact_string(),
    CommandDefinition {
      callback,
      attributes,
      options,
    },
  );

  match removed {
    Some(removed) => {}
    None => rv.set_undefined(),
  }
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
    let v = {
      let cmd = v8::Object::new(scope);

      // name
      let name_field = v8::String::new(scope, "name").unwrap();
      let name_value = v8::String::new(scope, name.as_ref()).unwrap();
      cmd.set(scope, name_field.into(), name_value.into());

      // attributes
      let attr_field = v8::String::new(scope, "attributes").unwrap();
      let attr_value = def.attributes.into_v8_object(scope);
      cmd.set(scope, attr_field.into(), attr_value.into());

      // options
      let opts_field = v8::String::new(scope, "options").unwrap();
      let opts_value = def.options.into_v8_object(scope);
      cmd.set(scope, opts_field.into(), opts_value.into());

      cmd
    };

    cmds.set_index(scope, i as u32, v.into());
  }

  rv.set(v8::Local::new(scope, cmds).into());
}

/// `Rsvim.cmd.remove` API.
pub fn remove(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let name = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.remove:{:?}", name);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow_mut();
  let mut commands = lock!(state.commands);
  commands.insert(name.to_compact_string(), (callback, attrs, opts));
}
