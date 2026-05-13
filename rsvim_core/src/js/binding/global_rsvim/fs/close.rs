//! Close file APIs.

use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;

pub fn fs_close<'s>(resource_table: ResourceTableArc, file_rid: ResourceId) {
  let mut resource_table = lock!(resource_table);
  let file_handle = resource_table.remove(&file_rid);
  debug_assert!(file_handle.is_some());
  // Drop file handle, i.e. close the file
}
