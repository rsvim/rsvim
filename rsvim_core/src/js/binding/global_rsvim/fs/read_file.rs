//! Read file APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::prelude::*;
use compact_str::ToCompactString;

pub fn fs_read_file(path: &Path) -> TheResult<Vec<u8>> {
  match std::fs::read(path) {
    Ok(buf) => Ok(buf),
    Err(e) => Err(TheErr::ReadFileByPathFailed(
      path.to_string_lossy().to_compact_string(),
      e,
    )),
  }
}

pub async fn async_fs_read_file(path: &Path) -> TheResult<Vec<u8>> {
  match tokio::fs::read(path).await {
    Ok(buf) => Ok(buf),
    Err(e) => Err(TheErr::ReadFileByPathFailed(
      path.to_string_lossy().to_compact_string(),
      e,
    )),
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
