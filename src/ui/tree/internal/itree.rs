//! Internal tree structure implementation: the `Itree` structure.

use crate::ui::tree::internal::inode::{Inode, InodeAttr, InodePtr};

#[derive(Debug, Clone)]
pub struct Itree<T> {
  root: Option<InodePtr<T>>,
}

impl<T> Itree<T> {
  pub fn new(root_value: T, root_attr: InodeAttr) -> Self {
    let node = Inode::new(None, root_value, root_attr);
    Itree {
      root: Some(Inode::ptr(node)),
    }
  }
}
