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
  pub fn new(loid: LayoutNodeId, shape: U16Rect) -> Self {
    Dummy {
      base: InodeBase::new(loid, shape),
    }
  }
}

inode_impl!(Dummy, base);

impl Widgetable for Dummy {}
