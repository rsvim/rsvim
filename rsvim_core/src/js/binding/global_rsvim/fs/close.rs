//! Close file APIs.

use crate::js::binding;
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
