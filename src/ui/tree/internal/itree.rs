//! Internal tree structure implementation: the `Itree` structure.

use crate::ui::tree::internal::inode::InodePtr;

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,
}
