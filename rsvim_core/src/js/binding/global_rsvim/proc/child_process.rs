//! Child process.

use crate::js::JsFuture;
use crate::js::binding;
use crate::prelude::*;
use compact_str::ToCompactString;

pub struct ReadTextFromChildProcessStdioFuture {
  pub promise: v8::Global<v8::PromiseResolver>,
  pub maybe_result: Option<TheResult<Vec<u8>>>,
}

impl JsFuture for ReadTextFromChildProcessStdioFuture {
  fn run(&mut self, scope: &mut v8::PinScope) {
    trace!("|ReadTextFromChildProcessStdioFuture|");

    let result = self.maybe_result.take().unwrap();

    // Handle when something goes wrong with read the stdio.
    if let Err(e) = result {
      let message = v8::String::new(scope, &e.to_string()).unwrap();
      let exception = v8::Exception::error(scope, message);
      binding::set_exception_code(scope, exception, &e);
      self.promise.open(scope).reject(scope, exception);
      return;
    }

    // Otherwise, resolve the promise passing the result.
    let data = result.unwrap();

    let bytes_read = v8::Integer::new(scope, data.len() as i32);

    self
      .promise
      .open(scope)
      .resolve(scope, bytes_read.into())
      .unwrap();
  }
}
