//! Close file APIs.

use crate::js::resource::ResourceTable;

pub fn fs_close<'s>(
  resource_table: &mut ResourceTable,
  scope: &mut v8::PinScope<'s, '_>,
  file_wrapper: v8::Local<'s, v8::Object>,
) {
  let file_resouce = 
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
