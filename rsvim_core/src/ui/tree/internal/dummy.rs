//! Common root node.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Root container.
pub struct Dummy {
  base: InodeBase,
}

impl Dummy {
  pub fn new(id: TreeNodeId, shape: U16Rect) -> Self {
    Dummy {
      base: InodeBase::new(id, shape),
    }
  }
}

inode_impl!(Dummy, base);

impl Widgetable for Dummy {}
