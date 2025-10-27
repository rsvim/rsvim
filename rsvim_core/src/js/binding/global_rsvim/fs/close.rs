//! Close file APIs.

use crate::get_cppgc_handle;
use crate::js::binding::global_rsvim::fs::handle;

pub fn fs_close<'s>(
  scope: &mut v8::PinScope<'s, '_>,
  file_wrapper: v8::Local<'s, v8::Object>,
) {
  if let Some(fd) = get_cppgc_handle!(scope, file_wrapper, Option<usize>).take()
  {
    // Note: By taking the file reference out of the option and immediately dropping
    // it will make rust to close the file.
    let file = handle::std_from_fd(fd);
    drop(file);
  } else {
    unreachable!();
  }
}
