//! `TextEncoder` APIs.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsTimerId;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use std::rc::Rc;

/// `TextEncoder.encode` API.
pub fn encode<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  // Get timer ID, and remove it.
  let timer_id = from_v8::<JsTimerId>(scope, args.get(0));
  let state_rc = JsRuntime::state(scope);

  let mut state = state_rc.borrow_mut();
  pending::remove_timer(&mut state, timer_id);
  trace!("|clear_timer| timer_id:{:?}", timer_id);
}
