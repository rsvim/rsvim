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

#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_builder::Builder)]
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

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with opening the file.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, get the result and deserialize it.
    let result = result.unwrap();

    // Deserialize bytes into a file-descriptor.
    let (fd, _fd_len) = bincode::decode_from_slice::<
      usize,
      bincode::config::Configuration,
    >(&result, bincode::config::standard())
    .unwrap();

    let file = util::from_fd(fd);

    let file_wrapper = v8::ObjectTemplate::new(scope);

    // Allocate space for the wrapped Rust type.
    file_wrapper.set_internal_field_count(2);

    let file_wrapper = file_wrapper.new_instance(scope).unwrap();
    let fd = v8::Number::new(scope, fd as f64);

    binding::set_constant_to(scope, file_wrapper, "fd", fd.into());

    let file_ptr =
      binding::set_internal_ref(scope, file_wrapper, 0, Some(file));
    let weak_rc = Rc::new(Cell::new(None));

    // Note: To automatically close the file (i.e., drop the instance) when
    // V8 garbage collects the object that internally holds the Rust file,
    // we use a Weak reference with a finalizer callback.
    let file_weak = v8::Weak::with_finalizer(
      scope,
      file_wrapper,
      Box::new({
        let weak_rc = weak_rc.clone();
        move |isolate| unsafe {
          drop(Box::from_raw(file_ptr));
          drop(v8::Weak::from_raw(isolate, weak_rc.get()));
        }
      }),
    );

    // Store the weak ref pointer into the "shared" cell.
    weak_rc.set(file_weak.into_raw());
    set_internal_ref(scope, file_wrapper, 1, weak_rc);

    self
      .promise
      .open(scope)
      .resolve(scope, file_wrapper.into())
      .unwrap();
  }
}
