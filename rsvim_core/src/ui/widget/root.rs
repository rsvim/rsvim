//! Root node.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Root node.
pub struct Root {
  base: ItreeRc,
  id: TreeNodeId,
}

impl Root {
  pub fn new(base: ItreeRc, id: TreeNodeId) -> Self {
    Root { base, id }
  }
}

inode_impl!(Root);

impl Widgetable for Root {}
