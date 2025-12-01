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
  pub fn new(lotree: ItreeWk, id: TreeNodeId) -> Self {
    Root {
      base: InodeBase::new(lotree, id),
    }
  }
}

inode_impl!(Root);

impl Widgetable for Root {}
