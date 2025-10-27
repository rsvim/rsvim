//! Read file APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::js::encdec::decode_bytes;
use crate::prelude::*;

pub fn fs_read(fd: usize, bufsize: usize) -> TheResult<Vec<u8>> {
  use std::io::Read;

  let mut file = handle::std_from_fd(fd);
  let mut buf: Vec<u8> = vec![0; bufsize];
  let n = match file.read(&mut buf) {
    Ok(n) => n,
    Err(e) => bail!(TheErr::ReadFileFailed(e)),
  };
  debug_assert!(n <= buf.capacity());
  unsafe {
    buf.set_len(n);
  }
  handle::std_to_fd(file);
  trace!("|fs_read| bufsize:{},n:{},buf:{:?}", bufsize, n, buf);

  Ok(buf)
}

pub async fn async_fs_read(fd: usize, bufsize: usize) -> TheResult<Vec<u8>> {
  use tokio::io::AsyncReadExt;

  let mut file = handle::tokio_from_fd(fd);
  let mut buf: Vec<u8> = Vec::with_capacity(bufsize);
  let n = match file.read(&mut buf).await {
    Ok(n) => n,
    Err(e) => bail!(TheErr::ReadFileFailed(e)),
  };
  debug_assert!(n <= buf.capacity());
  unsafe {
    buf.set_len(n);
  }
  handle::tokio_to_fd(file).await;
  trace!("|fs_read| bufsize:{},n:{},buf:{:?}", bufsize, n, buf);

  Ok(buf)
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
    let (result, _result_len) = decode_bytes::<FsReadResult>(&result.unwrap());
    let data = result.buf;
    let _read = result.read;

    debug_assert_eq!(_read, data.len());

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
