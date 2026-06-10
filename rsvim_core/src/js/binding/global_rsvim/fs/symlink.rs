//! File symbolic link.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::converter::*;
use crate::prelude::*;
use compact_str::ToCompactString;
use std::fs::Metadata;

#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  strum_macros::Display,
  strum_macros::EnumString,
)]
pub enum FsSymlinkOptions {
  #[strum(serialize = "file")]
  File,

  #[strum(serialize = "dir")]
  Dir,

  #[strum(serialize = "junction")]
  Junction,
}

#[cfg(target_family = "unix")]
pub fn fs_symlink(
  oldpath: &Path,
  newpath: &Path,
  options: FsSymlinkOptions,
) -> TheResult<()> {
  match std::os::unix::fs::symlink(oldpath, newpath) {
    Ok(_) => Ok(()),
    Err(e) => Err(TheErr::CreateSymlinkFailed(
      oldpath.to_string_lossy().to_compact_string(),
      newpath.to_string_lossy().to_compact_string(),
      e,
    )),
  }
}

#[cfg(target_family = "windows")]
pub fn fs_symlink(
  oldpath: &Path,
  newpath: &Path,
  options: FsSymlinkOptions,
) -> TheResult<()> {
  match options {
    FsSymlinkOptions::File => {
      match std::os::windows::fs::symlink_file(oldpath, newpath) {
        Ok(_) => Ok(()),
        Err(e) => Err(TheErr::CreateSymlinkFailed(
          oldpath.to_string_lossy().to_compact_string(),
          newpath.to_string_lossy().to_compact_string(),
          e,
        )),
      }
    }
    FsSymlinkOptions::Dir => {
      match std::os::windows::fs::symlink_dir(oldpath, newpath) {
        Ok(_) => Ok(()),
        Err(e) => Err(TheErr::CreateSymlinkFailed(
          oldpath.to_string_lossy().to_compact_string(),
          newpath.to_string_lossy().to_compact_string(),
          e,
        )),
      }
    }
    FsSymlinkOptions::Junction => match junction::create(oldpath, newpath) {
      Ok(_) => Ok(()),
      Err(e) => Err(TheErr::CreateSymlinkFailed(
        oldpath.to_string_lossy().to_compact_string(),
        newpath.to_string_lossy().to_compact_string(),
        e,
      )),
    },
  }
}

pub async fn async_fs_symlink(path: &Path) -> TheResult<()> {
  match tokio::fs::symlink_metadata(path).await {
    Ok(meta) => Ok(convert_metadata_to_fileinfo(meta)),
    Err(e) => Err(TheErr::ReadFileByPathFailed(
      path.to_string_lossy().to_compact_string(),
      e,
    )),
  }
}

pub struct FsStatFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsStatFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsStatFuture|");

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with it.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, get the result and deserialize it.
    let result = result.unwrap();

    // Deserialize bytes into file info.
    let file_info = postcard::from_bytes::<FsFileInfo>(&result).unwrap();
    let file_info = file_info.to_v8(scope);

    self.promise.open(scope).resolve(scope, file_info).unwrap();
  }
}
