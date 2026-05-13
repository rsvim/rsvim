//! APIs for `Rsvim.fs` namespace.

pub mod close;
pub mod fd;
pub mod open;
pub mod read;
pub mod read_file;
pub mod read_text_file;
pub mod write;

use crate::get_cppgc_handle;
use crate::is_v8_int;
use crate::is_v8_str;
use crate::js;
use crate::js::JsRuntime;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::close::fs_close;
use crate::js::binding::global_rsvim::fs::open::FsOpenFuture;
use crate::js::binding::global_rsvim::fs::open::FsOpenOptions;
use crate::js::binding::global_rsvim::fs::open::fs_open;
use crate::js::binding::global_rsvim::fs::read::FsReadFuture;
use crate::js::binding::global_rsvim::fs::read::fs_read;
use crate::js::binding::global_rsvim::fs::read_file::FsReadFileFuture;
use crate::js::binding::global_rsvim::fs::read_file::fs_read_file;
use crate::js::binding::global_rsvim::fs::read_text_file::FsReadTextFileFuture;
use crate::js::binding::global_rsvim::fs::read_text_file::fs_read_text_file;
use crate::js::binding::global_rsvim::fs::write::FsWriteFuture;
use crate::js::binding::global_rsvim::fs::write::fs_write;
use crate::js::converter::*;
use crate::js::pending;
use crate::js::resource::ResourceId;
use crate::prelude::*;
use itertools::Itertools;

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
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
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

  let state_rc = JsRuntime::state(scope);
  let resource_table = state_rc.borrow().resource_table.clone();

  let filename = Path::new(&filename);
  match fs_open(resource_table, filename, options) {
    Ok(file_rid) => {
      let file_rid = Into::<i32>::into(file_rid);
      let file_rid = file_rid.to_v8(scope);
      rv.set(file_rid.into());
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
  debug_assert!(is_v8_int!(args.get(0)));
  let file_rid = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  trace!("Rsvim.fs.close:{:?}", file_rid);
  let file_rid = ResourceId::from(file_rid);

  let state_rc = JsRuntime::state(scope);
  let resource_table = state_rc.borrow().resource_table.clone();

  fs_close(resource_table, file_rid);
}

/// `File.read` API.
pub fn read<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_int!(args.get(0)));
  let file_rid = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  let file_rid = ResourceId::from(file_rid);
  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  trace!("RsvimFs.read: {:?}, {:?}", file_rid, buf);

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
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  pending::create_fs_read(
    &mut state,
    task_id,
    fd,
    buf.byte_length(),
    Box::new(read_cb),
  );

  rv.set(promise.into());
}

/// `File.readSync` API.
pub fn read_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_int!(args.get(0)));
  let file_rid = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  let file_rid = ResourceId::from(file_rid);
  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  trace!("RsvimFs.readSync: {:?}, {:?}", file_rid, buf);

  let state_rc = JsRuntime::state(scope);
  let resource_table = state_rc.borrow().resource_table.clone();

  match fs_read(resource_table, file_rid, buf.byte_length()) {
    Ok(data) => {
      let buffer_store = buf.get_backing_store();
      for (i, b) in data.iter().enumerate() {
        buffer_store[i].set(*b);
      }
      rv.set_int32(data.len() as i32);
    }
    Err(e) => binding::throw_exception(scope, &e),
  }
}

/// `File.write` API.
pub fn write<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_int!(args.get(0)));
  let file_rid = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  let file_rid = ResourceId::from(file_rid);
  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  let buf = buf
    .get_backing_store()
    .iter()
    .map(|b| b.get())
    .collect_vec();
  trace!("RsvimFs.write: {:?}, {:?}", file_rid, buf);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let write_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = FsWriteFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  pending::create_fs_write(
    &mut state,
    task_id,
    file_rid,
    buf,
    Box::new(write_cb),
  );

  rv.set(promise.into());
}

/// `File.writeSync` API.
pub fn write_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 2);
  debug_assert!(is_v8_int!(args.get(0)));
  let file_rid = i32::from_v8(scope, args.get(0).to_integer(scope).unwrap());
  let file_rid = ResourceId::from(file_rid);
  debug_assert!(args.get(1).is_array_buffer());
  let buf = args.get(1).cast::<v8::ArrayBuffer>();
  let buf = buf
    .get_backing_store()
    .iter()
    .map(|b| b.get())
    .collect_vec();
  trace!("RsvimFs.writeSync: {:?}, {:?}", file_rid, buf);

  let state_rc = JsRuntime::state(scope);
  let resource_table = state_rc.borrow().resource_table.clone();

  match fs_write(resource_table, file_rid, buf) {
    Ok(bytes_written) => {
      rv.set_int32(bytes_written as i32);
    }
    Err(e) => binding::throw_exception(scope, &e),
  }
}

/// `Rsvim.fs.readFile` API.
pub fn read_file<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let filename = args.get(0).to_rust_string_lossy(scope);
  trace!("RsvimFs.readFile: {:?}", filename);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let read_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = FsReadFileFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  pending::create_fs_read_file(
    &mut state,
    task_id,
    Path::new(&filename),
    Box::new(read_cb),
  );

  rv.set(promise.into());
}

/// `Rsvim.fs.readFileSync` API.
pub fn read_file_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let filename = args.get(0).to_rust_string_lossy(scope);
  trace!("RsvimFs.readFileSync: {:?}", filename);

  match fs_read_file(Path::new(&filename)) {
    Ok(data) => {
      let buf = v8::ArrayBuffer::new(scope, data.len());
      let buffer_store = buf.get_backing_store();

      // Copy the slice's bytes into v8's typed-array backing store.
      for (i, b) in data.iter().enumerate() {
        buffer_store[i].set(*b);
      }

      rv.set(buf.into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
}

/// `Rsvim.fs.readTextFile` API.
pub fn read_text_file<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let filename = args.get(0).to_rust_string_lossy(scope);
  trace!("RsvimFs.readTextFile: {:?}", filename);

  let promise_resolver = v8::PromiseResolver::new(scope).unwrap();
  let promise = promise_resolver.get_promise(scope);

  let state_rc = JsRuntime::state(scope);
  let read_cb = {
    let promise = v8::Global::new(scope, promise_resolver);
    let state_rc = state_rc.clone();
    move |maybe_result: Option<TheResult<Vec<u8>>>| {
      let fut = FsReadTextFileFuture {
        promise: promise.clone(),
        maybe_result,
      };
      let mut state = state_rc.borrow_mut();
      state.pending_futures.push(Box::new(fut));
    }
  };

  let mut state = state_rc.borrow_mut();
  let task_id = js::TaskId::next();
  pending::create_fs_read_text_file(
    &mut state,
    task_id,
    Path::new(&filename),
    Box::new(read_cb),
  );

  rv.set(promise.into());
}

/// `Rsvim.fs.readTextFileSync` API.
pub fn read_text_file_sync<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  args: v8::FunctionCallbackArguments<'s>,
  mut rv: v8::ReturnValue,
) {
  debug_assert!(args.length() == 1);
  debug_assert!(is_v8_str!(args.get(0)));
  let filename = args.get(0).to_rust_string_lossy(scope);
  trace!("RsvimFs.readTextFileSync: {:?}", filename);

  match fs_read_text_file(Path::new(&filename)) {
    Ok(data) => {
      let data = v8::String::new(scope, &data).unwrap();

      rv.set(data.into());
    }
    Err(e) => {
      binding::throw_exception(scope, &e);
    }
  }
}
