//! APIs for `Rsvim.fs` namespace.

pub mod close;
pub mod handle;
pub mod open;
pub mod read;

use crate::create_cppgc_handle;
use crate::get_cppgc_handle;
use crate::is_v8_str;
use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::close::fs_close;
use crate::js::binding::global_rsvim::fs::close::fs_is_closed;
use crate::js::binding::global_rsvim::fs::open::FsOpenFuture;
use crate::js::binding::global_rsvim::fs::open::FsOpenOptions;
use crate::js::binding::global_rsvim::fs::open::fs_open;
use crate::js::binding::global_rsvim::fs::read::FsReadFuture;
use crate::js::binding::global_rsvim::fs::read::fs_read;
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
  debug_assert!(is_v8_str!(args.get(0)));
  let filename = args.get(0).to_rust_string_lossy(scope);
  debug_assert!(args.get(1).is_object());
  let options =
    FsOpenOptions::from_v8(scope, args.get(1).to_object(scope).unwrap());
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
    filename,
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
  let options =
    FsOpenOptions::from_v8(scope, args.get(1).to_object(scope).unwrap());
  trace!("Rsvim.fs.openSync:{:?} {:?}", filename, options);

  let filename = Path::new(&filename);
  match fs_open(filename, options) {
    Ok(fd) => {
      let file_wrapper = create_cppgc_handle!(scope, Some(fd), Option<usize>);
      rv.set(file_wrapper.into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
}

/// `Rsvim.fs.close` API.
pub fn close<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut _rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let file_wrapper = args.get(0);
  trace!("Rsvim.fs.close");

  fs_close(scope, file_wrapper.to_object(scope).unwrap());
}

/// `Rsvim.fs.isClosed` API.
pub fn is_closed<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  let file_wrapper = args.get(0);
  trace!("Rsvim.fs.isClosed");

  let result = fs_is_closed(scope, file_wrapper.to_object(scope).unwrap());
  rv.set_bool(result);
}

/// `File.read` API.
pub fn read<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(args.get(0).is_object());
  let file_wrapper = args.get(0).to_object(scope).unwrap();
  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  trace!("RsvimFs.read: {:?}, {:?}", file_wrapper, buf);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let read_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    let buffer_store = buf.get_backing_store().clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = FsReadFuture {
        promise: promise.clone(),
        buffer_store: buffer_store.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.insert(0, Box::new(fut));
    }
  };

  let fd = get_cppgc_handle!(scope, file_wrapper, Option<usize>)
    .as_ref()
    .unwrap();
  let mut state = state_rc.borrow_mut();
  let task_id = js::next_task_id();
  pending::create_fs_read(
    &mut state,
    task_id,
    *fd,
    buf.byte_length(),
    Box::new(read_cb),
  );

  rv.set(promise.into());
}
