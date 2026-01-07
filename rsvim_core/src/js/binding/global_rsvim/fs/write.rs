//! Write file APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::prelude::*;

pub fn fs_write(fd: usize, buf: Vec<u8>) -> TheResult<usize> {
  use std::io::Write;

  let mut file = handle::std_from_fd(fd);
  let n = match file.write(&buf) {
    Ok(n) => n,
    Err(e) => bail!(TheErr::WriteFileFailed(fd, e)),
  };
  debug_assert!(n <= buf.len());
  handle::std_to_fd(file);
  trace!("|fs_write| n:{},buf:{:?}", n, buf);

  Ok(n)
}

pub async fn async_fs_write(fd: usize, buf: Vec<u8>) -> TheResult<usize> {
  use tokio::io::AsyncWriteExt;

  let mut file = handle::tokio_from_fd(fd);
  let n = match file.write(&buf).await {
    Ok(n) => n,
    Err(e) => bail!(TheErr::WriteFileFailed(fd, e)),
  };
  debug_assert!(n <= buf.len());
  handle::tokio_to_fd(file).await;
  trace!("|async_fs_write| n:{},buf:{:?}", n, buf);

  Ok(n)
}

pub struct FsWriteFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsWriteFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsWriteFuture|");

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
    let bytes_written = bincode::deserialize::<usize>(&data).unwrap();

    let bytes_written = v8::Integer::new(scope, bytes_written as i32);

    self
      .promise
      .open(scope)
      .resolve(scope, bytes_written.into())
      .unwrap();
  }
}
