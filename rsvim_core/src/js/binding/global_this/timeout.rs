//! Timeout APIs.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsFutureId;
use crate::js::JsRuntime;
use crate::msg;
use crate::msg::MasterMessage;
use crate::prelude::*;
use std::rc::Rc;
use std::time::Duration;

struct TimeoutFuture {
  future_id: JsFutureId,
  cb: Rc<v8::Global<v8::Function>>,
  params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimeoutFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    trace!("|TimeoutFuture| run:{:?}", self.future_id);
    let undefined = v8::undefined(scope).into();
    let callback = v8::Local::new(scope, (*self.cb).clone());
    let args: Vec<v8::Local<v8::Value>> = self
      .params
      .iter()
      .map(|arg| v8::Local::new(scope, arg))
      .collect();

    let tc_scope = &mut v8::TryCatch::new(scope);

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

/// Javascript `setTimeout` API.
pub fn set_timeout(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  mut rv: v8::ReturnValue,
) {
  // Get timer's callback.
  let callback = v8::Local::<v8::Function>::try_from(args.get(0)).unwrap();
  let callback = Rc::new(v8::Global::new(scope, callback));

  // Get timer's expiration time in millis.
  let millis = args.get(1).int32_value(scope).unwrap() as u64;

  // Convert params argument (Array<Local<Value>>) to Rust vector.
  let params = match v8::Local::<v8::Array>::try_from(args.get(3)) {
    Ok(params) => (0..params.length()).fold(
      Vec::<v8::Global<v8::Value>>::new(),
      |mut acc, i| {
        let param = params.get_index(scope, i).unwrap();
        acc.push(v8::Global::new(scope, param));
        acc
      },
    ),
    Err(_) => vec![],
  };

  let state_rc = JsRuntime::state(scope);
  let mut state = state_rc.borrow_mut();
  let params = Rc::new(params);

  // Return timeout's internal id.
  let timer_id = js::next_timer_id();
  msg::sync_send_to_master(
    state.master_tx.clone(),
    MasterMessage::TimeoutReq(msg::TimeoutReq {
      timer_id,
      expire_at: Duration::from_millis(millis),
    }),
  );

  let timeout_cb = TimeoutFuture {
    future_id: timer_id,
    cb: Rc::clone(&callback),
    params: Rc::clone(&params),
  };
  state.pending_futures.insert(timer_id, Box::new(timeout_cb));
  state.timer_handles.insert(timer_id);
  rv.set(v8::Integer::new(scope, timer_id as i32).into());
  trace!("|set_timeout| timer_id:{:?}, millis:{:?}", timer_id, millis);
}

/// Javascript `clearTimeout` API.
pub fn clear_timeout(
  scope: &mut v8::HandleScope,
  args: v8::FunctionCallbackArguments,
  _: v8::ReturnValue,
) {
  // Get timer ID, and remove it.
  let timer_id = args.get(0).int32_value(scope).unwrap();
  let state_rc = JsRuntime::state(scope);

  state_rc.borrow_mut().timer_handles.remove(&timer_id);
  trace!("|clear_timeout| timer_id:{:?}", timer_id);
}
