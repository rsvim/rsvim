//! APIs for `Rsvim.fs` namespace.

pub mod fs_file;
pub mod open;

use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::open::FsOpenFuture;
use crate::js::binding::global_rsvim::fs::open::FsOpenOptions;
use crate::js::binding::global_rsvim::fs::open::fs_open;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;

/// `Rsvim.fs.open` API.
pub fn open<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  let filename = args.get(0).to_rust_string_lossy(scope);
  let options = FsOpenOptions::from_v8(scope, args.get(1));
  trace!("Rsvim.fs.open:{:?} {:?}", filename, options);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let open_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = FsOpenFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.insert(0, Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::next_task_id();
  let filename = Path::new(&filename);
  pending::create_fs_open(
    &mut state,
    task_id,
    &filename,
    options,
    Box::new(open_cb),
  );

  rv.set(promise.into());
}

/// `Rsvim.fs.openSync` API.
pub fn open_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  let filename = args.get(0).to_rust_string_lossy(scope);
  let options = FsOpenOptions::from_v8(scope, args.get(1));
  trace!("Rsvim.fs.open:{:?} {:?}", filename, options);

  let filename = Path::new(&filename);
  match fs_open(filename, options) {
    Ok(fd) => {
      let file_wrapper = v8::Object::new(scope);
      let fd_value = to_v8(scope, fd as f64);
      binding::set_constant_to(scope, file_wrapper, fs_file::FD, fd_value);
      rv.set(file_wrapper.into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
}
