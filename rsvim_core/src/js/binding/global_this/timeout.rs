//! Timeout APIs.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsTimerId;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use std::rc::Rc;

struct TimeoutFuture {
  cb: Rc<v8::Global<v8::Function>>,
  params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimeoutFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|TimeoutFuture| run");
    let undefined = v8::undefined(scope).into();
    let callback = v8::Local::new(scope, (*self.cb).clone());
    let args: Vec<v8::Local<v8::Value>> = self
      .params
      .iter()
      .map(|arg| v8::Local::new(scope, arg))
      .collect();

    v8::tc_scope!(let tc_scope, scope);

    callback.call(tc_scope, undefined, &args);

    // Report if callback threw an exception.
    if tc_scope.has_caught() {
      let exception = tc_scope.exception().unwrap();
      let exception = v8::Global::new(tc_scope, exception);
      let state_rc = JsRuntime::state(tc_scope);
      state_rc
        .borrow_mut()
        .exceptions
        .capture_exception(exception);
    }
  }
}

/// Javascript `setTimeout`/`setInterval` API.
pub fn create_timer<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() >= 3);

  // Get timer's callback.
  let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
  let callback = Rc::new(v8::Global::new(scope, callback));

  // Get timer's delay time in millis.
  let delay = from_v8::<u32>(scope, args.get(1)).unwrap();
  // Get timer's repeated.
  let repeated = from_v8::<bool>(scope, args.get(2)).unwrap();

  // NOTE: Don't delete this part of code, it shows how to convert function
  // arguments into an array of values.
  // Convert params argument (Array<Local<Value>>) to Rust vector.
  // let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
  //   Ok(params) => (0..params.length()).fold(
  //     Vec::<v8::Global<v8::Value>>::new(),
  //     |mut acc, i| {
  //       let param = params.get_index(scope, i).unwrap();
  //       acc.push(v8::Global::new(scope, param));
  //       acc
  //     },
  //   ),
  //   Err(_) => vec![],
  // };
  let params = vec![];

  let params = Rc::new(params);

  let state_rc = JsRuntime::state(scope);
  let timer_cb = {
    let state_rc = state_rc.clone();
    move || {
      let fut = TimeoutFuture {
        cb: Rc::clone(&callback),
        params: Rc::clone(&params),
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.insert(0, Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let timer_id = js::next_timer_id();
  pending::create_timer(
    &mut state,
    timer_id,
    delay as u64,
    repeated,
    Box::new(timer_cb),
  );
  rv.set(to_v8(scope, timer_id).unwrap());
  trace!(
    "|create_timer| timer_id:{:?}, delay:{:?}, repeated:{:?}",
    timer_id, delay, repeated
  );
}

/// Javascript `clearTimeout`/`clearInterval` API.
pub fn clear_timer<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  _: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  // Get timer ID, and remove it.
  let timer_id = from_v8::<JsTimerId>(scope, args.get(0)).unwrap();
  let state_rc = JsRuntime::state(scope);

  let mut state = state_rc.borrow_mut();
  pending::remove_timer(&mut state, timer_id);
  trace!("|clear_timer| timer_id:{:?}", timer_id);
}
