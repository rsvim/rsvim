//! Close file APIs.

use crate::get_cppgc_handle;
use crate::js::binding;
use crate::js::binding::global_rsvim::fs::handle;
use crate::prelude::*;
use parking_lot::Mutex;
use std::fs::File;
use std::sync::Arc;

pub fn fs_close<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  file_wrapper: v8::Local<'s, v8::Object>,
) {
  if let Some(file) =
    get_cppgc_handle!(scope, file_wrapper, Option<Arc<Mutex<File>>>).take()
  {
    // Note: By taking the file reference out of the option and immediately dropping
    // it will make rust to close the file.
    drop(file);
  } else {
    binding::throw_exception(scope, &TheErr::FileAlreadyClosed);
  }
}

pub fn fs_is_closed<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  file_wrapper: v8::Local<'s, v8::Object>,
) -> bool {
  binding::get_internal_ref::<Option<File>>(scope, file_wrapper, 0).is_none()
}
