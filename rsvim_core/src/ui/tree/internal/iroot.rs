//! Common root node.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Root container.
pub struct Root {
  base: InodeBase,
}

impl Root {
  pub fn new(loid: LayoutNodeId, shape: U16Rect) -> Self {
    Root {
      base: InodeBase::new(loid, shape),
    }
  }
}

inode_impl!(Root, base);

impl Widgetable for Root {}
