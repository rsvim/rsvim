//! Root node.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Root node.
pub struct Root {
  base: InodeBase,
}

impl Root {
  pub fn new(id: TreeNodeId, shape: U16Rect) -> Self {
    Root {
      base: InodeBase::new(id, shape),
    }
  }
}

inode_impl!(Root, base);

impl Widgetable for Root {}
