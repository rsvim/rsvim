//! APIs for `Rsvim.fs` namespace.

pub mod open;
pub mod util;

use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::JsTimerId;
use crate::js::binding;
use crate::js::command::def::CommandDefinition;
use crate::js::converter::*;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::CompactString;
use compact_str::ToCompactString;
use ringbuf::traits::RingBuffer;
use std::fs;
use std::fs::File;
use std::rc::Rc;

/// `Rsvim.fs.open` API.
pub fn open(
  scope: &mut v8::PinScope,
  args: v8::FunctionCallbackArguments,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let message = args.get(0).to_rust_string_lossy(scope);
  trace!("Rsvim.cmd.echo:{:?}", message);

  let state_rc = JsRuntime::state(scope);
  let state = state_rc.borrow();
}
