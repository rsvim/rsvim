//! APIs for `Rsvim.fs` namespace.

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

#[cfg(not(target_family = "windows"))]
fn to_fd(file: File) -> usize {
  use std::os::fd::IntoRawFd;
  file.into_raw_fd() as usize
}

#[cfg(target_family = "windows")]
fn to_fd(file: File) -> usize {
  use std::os::windows::io::IntoRawHandle;
  file.into_raw_handle() as usize
}

#[cfg(not(target_family = "windows"))]
fn from_fd(fd: usize) -> File {
  use std::os::fd::FromRawFd;
  unsafe { File::from_raw_fd(fd as std::os::fd::RawFd) }
}

#[cfg(target_family = "windows")]
fn from_fd(handle: usize) -> File {
  use std::os::windows::io::FromRawHandle;
  unsafe { File::from_raw_handle(handle as std::os::windows::io::RawHandle) }
}

struct FsOpenFuture {
  promise: v8::Global<v8::PromiseResolver>,
  maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsOpenFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsOpenFuture| run");
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
