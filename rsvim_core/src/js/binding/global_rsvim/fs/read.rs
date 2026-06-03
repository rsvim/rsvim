//! Read APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::resource::Resource;
use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;

pub fn fs_read(
  resource_table: ResourceTableArc,
  rid: ResourceId,
  bufsize: usize,
) -> TheResult<Vec<u8>> {
  use std::io::Read;

  let res = lock!(resource_table).get(&rid).cloned();
  debug_assert!(res.is_some());
  match res.unwrap() {
    Resource::File(res) => {
      let handle = res.data();
      let mut handle = lock!(handle);
      let mut buf: Vec<u8> = vec![0; bufsize];
      let n = match handle.read(&mut buf) {
        Ok(n) => n,
        Err(e) => {
          return Err(TheErr::ReadFileByRidFailed(rid, e));
        }
      };
      debug_assert!(n <= buf.capacity());
      unsafe {
        buf.set_len(n);
      }
      trace!("|fs_read| bufsize:{},n:{},buf:{:?}", bufsize, n, buf);

      Ok(buf)
    }
    _ => unreachable!(),
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
