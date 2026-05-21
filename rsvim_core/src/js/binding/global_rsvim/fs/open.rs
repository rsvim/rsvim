//! Open file APIs.

use crate::from_v8_prop;
use crate::js::JsFuture;
use crate::js::binding;
use crate::js::converter::*;
use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;
use compact_str::ToCompactString;

// Attribute names.
pub const APPEND: &str = "append";
pub const CREATE: &str = "create";
pub const CREATE_NEW: &str = "createNew";
pub const READ: &str = "read";
pub const TRUNCATE: &str = "truncate";
pub const WRITE: &str = "write";

// Default values.
pub const APPEND_DEFAULT: bool = false;
pub const CREATE_DEFAULT: bool = false;
pub const CREATE_NEW_DEFAULT: bool = false;
pub const READ_DEFAULT: bool = false;
pub const TRUNCATE_DEFAULT: bool = false;
pub const WRITE_DEFAULT: bool = false;

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  derive_builder::Builder,
  rsvim_macro::ToV8,
  rsvim_macro::FromV8,
)]
// See: <https://doc.rust-lang.org/std/fs/struct.OpenOptions.html>.
pub struct FsOpenOptions {
  #[builder(default = APPEND_DEFAULT)]
  #[from_v8_bool]
  pub append: bool,

  #[builder(default = CREATE_DEFAULT)]
  #[from_v8_bool]
  pub create: bool,

  #[builder(default = CREATE_NEW_DEFAULT)]
  #[from_v8_bool]
  pub create_new: bool,

  #[builder(default = READ_DEFAULT)]
  #[from_v8_bool]
  pub read: bool,

  #[builder(default = TRUNCATE_DEFAULT)]
  #[from_v8_bool]
  pub truncate: bool,

  #[builder(default = WRITE_DEFAULT)]
  #[from_v8_bool]
  pub write: bool,
}

pub fn fs_open(
  resource_table: ResourceTableArc,
  path: &Path,
  opts: FsOpenOptions,
) -> TheResult<ResourceId> {
  match std::fs::OpenOptions::new()
    .append(opts.append)
    .create(opts.create)
    .create_new(opts.create_new)
    .read(opts.read)
    .truncate(opts.truncate)
    .write(opts.write)
    .open(path)
  {
    Ok(file) => {
      let mut resource_table = lock!(resource_table);
      Ok(resource_table.add_file(file))
    }
    Err(e) => Err(TheErr::OpenFileFailed(
      path.to_string_lossy().to_compact_string(),
      e,
    )),
  }
}

pub async fn async_fs_open(
  resource_table: ResourceTableArc,
  path: &Path,
  opts: FsOpenOptions,
) -> TheResult<ResourceId> {
  match tokio::fs::OpenOptions::new()
    .append(opts.append)
    .create(opts.create)
    .create_new(opts.create_new)
    .read(opts.read)
    .truncate(opts.truncate)
    .write(opts.write)
    .open(path)
    .await
  {
    Ok(file) => {
      let file = file.into_std().await;
      let mut resource_table = lock!(resource_table);
      Ok(resource_table.add_file(file))
    }
    Err(e) => Err(TheErr::OpenFileFailed(
      path.to_string_lossy().to_compact_string(),
      e,
    )),
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
    let file_rid = postcard::from_bytes::<ResourceId>(&result).unwrap();
    let file_rid = Into::<i32>::into(file_rid);
    let file_rid = file_rid.to_v8(scope);

    self.promise.open(scope).resolve(scope, file_rid).unwrap();
  }
}
