//! Read file APIs.

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
use std::cell::Cell;
use std::fs::File;
use std::rc::Rc;

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
    let result = result.unwrap();

    // Deserialize bytes into a file-descriptor.
    let (data, data_len) = decode_bytes::<Vec<u8>>(&result);
    debug_assert_eq!(data.len(), data_len);

    // Copy the slice's bytes into v8's typed-array backing store.
    for (i, b) in data.iter().enumerate() {
      self.buffer_store[i].set(*b);
    }

    let bytes_read = v8::Integer::new(scope, data_len as i32);

    self
      .promise
      .open(scope)
      .resolve(scope, bytes_read.into())
      .unwrap();
  }
}
