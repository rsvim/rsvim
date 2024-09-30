//! APIs for `globalThis` namespace.
//! See WinterCG standard: <https://github.com/wintercg/proposal-common-minimum-api>

use crate::glovar;
use crate::js::binding::set_function_to;
use crate::js::JsRuntime;

use std::rc::Rc;
use std::time::Duration;
use tracing::debug;

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

  let timeout_cb = {
    let state_rc = state_rc.clone();
    move |_: LoopHandle| {
      let mut state = state_rc.borrow_mut();
      let future = TimeoutFuture {
        cb: Rc::clone(&callback),
        params: Rc::clone(&params),
      };
      state.pending_futures.push(Box::new(future));

      // Note: It's important to send an interrupt signal to the event-loop to prevent the
      // event-loop from idling in the poll phase, waiting for I/O, while the timer's JS
      // future is ready in the runtime level.
      if !state.wake_event_queued {
        state.interrupt_handle.interrupt();
        state.wake_event_queued = true;
      }
    }
  };

  // Schedule a new timer to the event-loop.
  let state = state_rc.borrow();
  let id = state.handle.timer(millis, repeatable, timeout_cb);

  // Return timeout's internal id.
  rv.set(v8::Number::new(scope, id as f64).into());
}

pub fn clear_timeout() {}
