//! `Rsvim.fs.open` and `Rsvim.fs.openSync` APIs.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::js::converter::*;
use crate::js::encdec::decode_bytes;
use crate::prelude::*;
use std::cell::Cell;
use std::fs::File;
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

pub const APPEND: &str = "append";
pub const CREATE: &str = "create";
pub const CREATE_NEW: &str = "create_new";
pub const READ: &str = "read";
pub const TRUNCATE: &str = "truncate";
pub const WRITE: &str = "write";

pub const APPEND_DEFAULT: bool = false;
pub const CREATE_DEFAULT: bool = false;
pub const CREATE_NEW_DEFAULT: bool = false;
pub const READ_DEFAULT: bool = false;
pub const TRUNCATE_DEFAULT: bool = false;
pub const WRITE_DEFAULT: bool = false;

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

impl FromV8 for FsOpenOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
  ) -> Self {
    let mut builder = FsOpenOptionsBuilder::default();
    let obj = value.to_object(scope).unwrap();

    // append
    let append_name = to_v8(scope, APPEND);
    if let Some(append_value) = obj.get(scope, append_name) {
      builder.append(from_v8::<bool>(scope, append_value));
    }

    // create
    let create_name = to_v8(scope, CREATE);
    if let Some(create_value) = obj.get(scope, create_name) {
      builder.create(from_v8::<bool>(scope, create_value));
    }

    // create_new
    let create_new_name = to_v8(scope, CREATE_NEW);
    if let Some(create_new_value) = obj.get(scope, create_new_name) {
      builder.create_new(from_v8::<bool>(scope, create_new_value));
    }

    // read
    let read_name = to_v8(scope, READ);
    if let Some(read_value) = obj.get(scope, read_name) {
      builder.read(from_v8::<bool>(scope, read_value));
    }

    // truncate
    let truncate_name = to_v8(scope, TRUNCATE);
    if let Some(truncate_value) = obj.get(scope, truncate_name) {
      builder.truncate(from_v8::<bool>(scope, truncate_value));
    }

    // write
    let write_name = to_v8(scope, WRITE);
    if let Some(write_value) = obj.get(scope, write_name) {
      builder.write(from_v8::<bool>(scope, write_value));
    }

    builder.build().unwrap()
  }
}

impl ToV8 for FsOpenOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Value> {
    let obj = v8::Object::new(scope);

    // append
    let append_value = to_v8(scope, self.append());
    binding::set_property_to(scope, obj, APPEND, append_value);

    // create
    let create_value = to_v8(scope, self.create());
    binding::set_property_to(scope, obj, CREATE, create_value);

    // create_new
    let create_new_value = to_v8(scope, self.create_new());
    binding::set_property_to(scope, obj, CREATE_NEW, create_new_value);

    // read
    let read_value = to_v8(scope, self.read());
    binding::set_property_to(scope, obj, READ, read_value);

    // truncate
    let truncate_value = to_v8(scope, self.truncate());
    binding::set_property_to(scope, obj, TRUNCATE, truncate_value);

    // write
    let write_value = to_v8(scope, self.write());
    binding::set_property_to(scope, obj, WRITE, write_value);

    obj.into()
  }
}

pub fn create_fs_file_wrapper<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  fd: usize,
) -> v8::Local<'s, v8::Object> {
  let file_handle = handle::from_fd(fd);

  let file_wrapper = v8::ObjectTemplate::new(scope);

  // Allocate internal field for the wrapped `std::fs::File`:
  // 0-index: The `file_handle`, i.e. the `std::fs::File`
  // 1-index: The `file_weak` finalizer, it helps release the `file_handle`
  file_wrapper.set_internal_field_count(3);
  let file_wrapper = file_wrapper.new_instance(scope).unwrap();

  let file_ptr = binding::set_internal_ref::<Option<File>>(
    scope,
    file_wrapper,
    0,
    Some(file_handle),
  );
  let weak_rc = Rc::new(Cell::new(None));

  // To automatically drop the file_handle instance when
  // V8 garbage collects the object that internally holds the Rust instance,
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
  binding::set_internal_ref(scope, file_wrapper, 1, weak_rc);

  file_wrapper
}

pub fn fs_open(path: &Path, opts: FsOpenOptions) -> TheResult<usize> {
  match std::fs::OpenOptions::new()
    .append(opts.append())
    .create(opts.create())
    .create_new(opts.create_new())
    .read(opts.read())
    .truncate(opts.truncate())
    .write(opts.write())
    .open(path)
  {
    Ok(file) => Ok(handle::to_fd(file)),
    Err(e) => bail!(TheErr::OpenFileFailed(
      path.to_string_lossy().to_string(),
      e
    )),
  }
}

pub async fn async_fs_open(
  path: &Path,
  opts: FsOpenOptions,
) -> TheResult<usize> {
  match tokio::fs::OpenOptions::new()
    .append(opts.append())
    .create(opts.create())
    .create_new(opts.create_new())
    .read(opts.read())
    .truncate(opts.truncate())
    .write(opts.write())
    .open(path)
    .await
  {
    Ok(file) => {
      let file = file.into_std().await;
      Ok(handle::to_fd(file))
    }
    Err(e) => bail!(TheErr::OpenFileFailed(
      path.to_string_lossy().to_string(),
      e
    )),
  }
}

pub struct FsOpenFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
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
    let (fd, _fd_len) = decode_bytes::<usize>(&result);
    let file_wrapper = create_fs_file_wrapper(scope, fd);

    self
      .promise
      .open(scope)
      .resolve(scope, file_wrapper.into())
      .unwrap();
  }
}
