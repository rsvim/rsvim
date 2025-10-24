//! Read file APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::prelude::*;

pub fn fs_read(fd: usize, bufsize: usize) -> TheResult<(Vec<u8>, usize)> {
  use std::io::Read;

  let mut file = handle::std_from_fd(fd);
  let mut buf: Vec<u8> = Vec::with_capacity(bufsize);
  match file.read(&mut buf) {
    Ok(n) => Ok((buf, n)),
    Err(e) => bail!(TheErr::ReadFileFailed(e)),
  }
}

pub async fn async_fs_read(
  fd: usize,
  bufsize: usize,
) -> TheResult<(Vec<u8>, usize)> {
  use tokio::io::AsyncReadExt;

  let mut file = handle::tokio_from_fd(fd);
  let mut buf: Vec<u8> = Vec::with_capacity(bufsize);
  match file.read(&mut buf).await {
    Ok(n) => Ok((buf, n)),
    Err(e) => bail!(TheErr::ReadFileFailed(e)),
  }
}

pub struct FsReadFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub buffer_store: v8::SharedRef<v8::BackingStore>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsReadFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsReadFuture|");

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with opening the file.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, resolve the promise passing the result.
    let data = result.unwrap();

    // Copy the slice's bytes into v8's typed-array backing store.
    for (i, b) in data.iter().enumerate() {
      self.buffer_store[i].set(*b);
    }

    let bytes_read = v8::Integer::new(scope, data.len() as i32);

    self
      .promise
      .open(scope)
      .resolve(scope, bytes_read.into())
      .unwrap();
  }
}
