//! `Rsvim.fs.close` and `Rsvim.fs.closeSync` APIs.

use crate::js::JsFuture;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::js::converter::*;
use crate::js::encdec::decode_bytes;
use crate::prelude::*;
use std::fs::File;

pub fn fs_close<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  file_wrapper: v8::Local<'s, v8::Object>,
) {
  if let Some(file) =
    binding::get_internal_ref::<Option<File>>(scope, file_wrapper, 0).take()
  {
    // Note: By taking the file reference out of the option and immediately dropping
    // it will make rust to close the file.
    drop(file);
  } else {
    binding::throw_exception(scope, &TheErr::FileAlreadyClosed);
  }
}

pub struct FsCloseFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for FsCloseFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|FsCloseFuture| run");

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

    let file_handle_wrapper = v8::Object::new(scope);
    let fd_value = to_v8(scope, fd as f64);
    binding::set_constant_to(scope, file_handle_wrapper, handle::FD, fd_value);

    self
      .promise
      .open(scope)
      .resolve(scope, file_handle_wrapper.into())
      .unwrap();
  }
}
