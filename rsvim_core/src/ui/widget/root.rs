//! Root node.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Root node.
pub struct Root {
  base: IrelationshipRc,
  id: TreeNodeId,
}

impl Root {
  pub fn new(base: IrelationshipRc, id: TreeNodeId) -> Self {
    Root { base, id }
  }
}

inode_impl!(Root);

impl Widgetable for Root {}
