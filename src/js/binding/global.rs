//! APIs for `globalThis` namespace.
//! See WinterCG standard: <https://github.com/wintercg/proposal-common-minimum-api>

use crate::glovar;
use crate::js::binding::set_function_to;
use crate::js::msg::{Dummy, JsRuntimeToEventLoopMessage};
use crate::js::{JsFuture, JsRuntime};
use crate::uuid;

use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;
use tracing::debug;

struct TimeoutFuture {
  cb: Rc<v8::Global<v8::Function>>,
  params: Rc<Vec<v8::Global<v8::Value>>>,
}

impl JsFuture for TimeoutFuture {
  fn run(&mut self, scope: &mut v8::HandleScope) {
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

  // Decide if the timer is an interval.
  let repeatable = args.get(2).to_rust_string_lossy(scope) == "true";

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
  let params = Rc::new(params);
  let join_handle = state_rc.borrow().task_tracker.spawn(async move {
    tokio::time::sleep(Duration::from_millis(millis)).await;
    let undefined = v8::undefined(raw_scope.as_mut()).into();
    let tc_scope = &mut v8::TryCatch::new(raw_scope.as_mut());

    let callback = v8::Local::new(scope, (*callback).clone());
    let args: Vec<v8::Local<v8::Value>> = params
      .iter()
      .map(|arg| v8::Local::new(scope, arg))
      .collect();
    callback.call(tc_scope, undefined, &args);

    // Report if callback threw an exception.
    if tc_scope.has_caught() {
      let exception = tc_scope.exception().unwrap();
      let exception = v8::Global::new(tc_scope, exception);
      let state = JsRuntime::state(tc_scope);
      state.borrow_mut().exceptions.capture_exception(exception);
    }

    state_rc
      .borrow()
      .js_worker_send_to_master
      .send(JsRuntimeToEventLoopMessage::Dummy(Dummy::default()))
      .await;
  });

  let job_id = uuid::next();

  // Return timeout's internal id.
  rv.set(v8::Number::new(scope, job_id as f64).into());
}

pub fn clear_timeout() {}
