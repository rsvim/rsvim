//! `Rsvim.fs.open` and `Rsvim.fs.openSync` APIs.

use crate::buf::opt;
use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::JsTimerId;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::util;
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
use std::fs::OpenOptions;
use std::rc::Rc;

// See: <https://doc.rust-lang.org/std/fs/struct.OpenOptions.html>.
flags_impl!(
  FsOpenOptionFlags,
  u8,
  APPEND,
  CREATE,
  CREATE_NEW,
  READ,
  TRUNCATE,
  WRITE
);

// All flags are false
const FS_OPEN_OPTION_FLAGS: FsOpenOptionFlags = FsOpenOptionFlags::empty();

#[derive(Debug, Clone, PartialEq, Eq, derive_builder::Builder)]
pub struct FsOpenOptions {
  #[builder(default = FS_OPEN_OPTION_FLAGS)]
  #[builder(setter(custom))]
  // append
  // create
  // create_new
  // read
  // truncate
  // write
  flags: FsOpenOptionFlags,
}

flags_builder_impl!(
  FsOpenOptionsBuilder,
  flags,
  FsOpenOptionFlags,
  append,
  create,
  create_new,
  read,
  truncate,
  write
);

impl FsOpenOptions {
  pub fn append(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::APPEND)
  }

  pub fn create(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::CREATE)
  }

  pub fn create_new(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::CREATE_NEW)
  }

  pub fn read(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::READ)
  }

  pub fn truncate(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::TRUNCATE)
  }

  pub fn write(&self) -> bool {
    self.flags.contains(FsOpenOptionFlags::WRITE)
  }
}

fn open_file_impl(path: &Path, opts: FsOpenOptions) -> TheResult<usize> {
  match OpenOptions::new()
    .append(opts.append())
    .create(opts.create())
    .create_new(opts.create_new())
    .read(opts.read())
    .truncate(opts.truncate())
    .write(opts.write())
    .open(path)
  {
    Ok(file) => Ok(util::to_fd(file)),
    Err(e) => bail!(TheErr::OpenFileFailed(
      path.to_string_lossy().to_string(),
      e
    )),
  }
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
