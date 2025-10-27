//! Open file APIs.

use crate::flags_builder_impl;
use crate::flags_impl;
use crate::from_v8_prop;
use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::js::converter::*;
use crate::js::encdec::decode_bytes;
use crate::prelude::*;
use crate::to_v8_prop;
use crate::wrap_cppgc_handle;

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
pub const CREATE_NEW: &str = "createNew";
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
  fs_open_option_flags: FsOpenOptionFlags,
}

impl FsOpenOptionsBuilder {
  flags_builder_impl!(fs_open_option_flags, append);
  flags_builder_impl!(fs_open_option_flags, create);
  flags_builder_impl!(fs_open_option_flags, create_new);
  flags_builder_impl!(fs_open_option_flags, read);
  flags_builder_impl!(fs_open_option_flags, truncate);
  flags_builder_impl!(fs_open_option_flags, write);
}

impl FsOpenOptions {
  pub fn append(&self) -> bool {
    self
      .fs_open_option_flags
      .contains(FsOpenOptionFlags::APPEND)
  }

  pub fn create(&self) -> bool {
    self
      .fs_open_option_flags
      .contains(FsOpenOptionFlags::CREATE)
  }

  pub fn create_new(&self) -> bool {
    self
      .fs_open_option_flags
      .contains(FsOpenOptionFlags::CREATE_NEW)
  }

  pub fn read(&self) -> bool {
    self.fs_open_option_flags.contains(FsOpenOptionFlags::READ)
  }

  pub fn truncate(&self) -> bool {
    self
      .fs_open_option_flags
      .contains(FsOpenOptionFlags::TRUNCATE)
  }

  pub fn write(&self) -> bool {
    self.fs_open_option_flags.contains(FsOpenOptionFlags::WRITE)
  }
}

impl StructFromV8 for FsOpenOptions {
  fn from_v8<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    obj: v8::Local<'s, v8::Object>,
  ) -> Self {
    let mut builder = FsOpenOptionsBuilder::default();

    from_v8_prop!(builder, obj, scope, bool, append);
    from_v8_prop!(builder, obj, scope, bool, create);
    from_v8_prop!(builder, obj, scope, bool, create_new);
    from_v8_prop!(builder, obj, scope, bool, read);
    from_v8_prop!(builder, obj, scope, bool, truncate);
    from_v8_prop!(builder, obj, scope, bool, write);

    builder.build().unwrap()
  }
}

impl StructToV8 for FsOpenOptions {
  fn to_v8<'s>(
    &self,
    scope: &mut v8::PinScope<'s, '_>,
  ) -> v8::Local<'s, v8::Object> {
    let obj = v8::Object::new(scope);

    to_v8_prop!(self, obj, scope, append, ());
    to_v8_prop!(self, obj, scope, create, ());
    to_v8_prop!(self, obj, scope, create_new, ());
    to_v8_prop!(self, obj, scope, read, ());
    to_v8_prop!(self, obj, scope, truncate, ());
    to_v8_prop!(self, obj, scope, write, ());

    obj
  }
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
    Ok(file) => Ok(handle::std_to_fd(file)),
    Err(e) => bail!(TheErr::IoErr(e)),
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
      let fd = handle::tokio_to_fd(file).await;
      Ok(fd)
    }
    Err(e) => bail!(TheErr::IoErr(e)),
  }
}

pub struct FsOpenFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsOpenFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsOpenFuture|");

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
    let file_wrapper = wrap_cppgc_handle!(scope, Some(fd), Option<usize>);

    self
      .promise
      .open(scope)
      .resolve(scope, file_wrapper.into())
      .unwrap();
  }
}
