//! Write file APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::resource::Resource;
use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;

pub fn fs_write(
  resource_table: ResourceTableArc,
  rid: ResourceId,
  buf: Vec<u8>,
) -> TheResult<usize> {
  use std::io::Write;

  let res = lock!(resource_table).get(&rid).cloned();
  debug_assert!(res.is_some());
  match res.unwrap() {
    Resource::File(res) => {
      let handle = res.data();
      let mut handle = lock!(handle);
      let n = match handle.write(&buf) {
        Ok(n) => n,
        Err(e) => return Err(TheErr::WriteFileByRidFailed(rid, e)),
      };
      debug_assert!(n <= buf.len());
      trace!("|fs_write| n:{},buf:{:?}", n, buf);

      Ok(n)
    }
    _ => unreachable!(),
  }
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
    let bytes_written = postcard::from_bytes::<usize>(&data).unwrap();

    let bytes_written = v8::Integer::new(scope, bytes_written as i32);

    self
      .promise
      .open(scope)
      .resolve(scope, bytes_written.into())
      .unwrap();
  }
}
