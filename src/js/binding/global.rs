//! APIs for `globalThis` namespace.
//! See WinterCG standard: <https://github.com/wintercg/proposal-common-minimum-api>

use crate::glovar;
use crate::js::binding::set_function_to;
use crate::js::msg::{self as jsmsg, JsRuntimeToEventLoopMessage};
use crate::js::{self, JsFuture, JsFutureId, JsRuntime};

use std::rc::Rc;
use std::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tracing::debug;

struct TimeoutFuture {
  future_id: JsFutureId,
  cb: Rc<v8::Global<v8::Function>>,
  params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimeoutFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
    debug!("set_timeout callback:{:?}", self.future_id);
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
      let state = JsRuntime::state(tc_scope);
      state.borrow_mut().exceptions.capture_exception(exception);
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
    Ok(params) => (0..params.length()).fold(Vec::<v8::Global<v8::Value>>::new(), |mut acc, i| {
      let param = params.get_index(scope, i).unwrap();
      acc.push(v8::Global::new(scope, param));
      acc
    }),
    Err(_) => vec![],
  };

  let state_rc = JsRuntime::state(scope);
  let state_rc2 = Rc::clone(&state_rc);
  let mut state = state_rc.borrow_mut();
  let params = Rc::new(params);

  // Return timeout's internal id.
  let timer_id = js::next_future_id();
  let js_runtime_send_to_master = state_rc.borrow().js_runtime_send_to_master.clone();
  let current_handle = tokio::runtime::Handle::current();
  current_handle.spawn_blocking(move || {
    let _ = js_runtime_send_to_master.blocking_send(JsRuntimeToEventLoopMessage::TimeoutReq(
      jsmsg::TimeoutReq::new(timer_id, Duration::from_millis(millis)),
    ));
  });
  let timeout_cb = TimeoutFuture {
    future_id: timer_id,
    cb: Rc::clone(&callback),
    params: Rc::clone(&params),
  };
  state.pending_futures.insert(timer_id, Box::new(timeout_cb));
  state.timeout_handles.insert(timer_id);
  rv.set(v8::Number::new(scope, timer_id as f64).into());
  debug!("set_timeout:{:?}, millis:{:?}", timer_id, millis);
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

  state_rc.borrow_mut().timeout_handles.remove(&timer_id);
  debug!("clear_timeout: {:?}", timer_id);
}
