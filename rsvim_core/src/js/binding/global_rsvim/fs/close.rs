//! Close file APIs.

use crate::js::resource::ResourceId;
use crate::js::resource::ResourceTableArc;
use crate::prelude::*;

pub fn fs_close<'s>(resource_table: ResourceTableArc, rid: ResourceId) {
  let mut resource_table = lock!(resource_table);
  let mut handle = resource_table.remove(&rid);
  debug_assert!(handle.is_some());
  // Drop file handle, i.e. close the file
  handle.take();
}
