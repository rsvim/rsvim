//! Root container is the root node in the widget tree.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone, Copy)]
/// Logical node that renders nothing but give a cerntain shape for its
/// descendant nodes.
pub struct Panel {
  base: InodeBase,
}

impl Panel {
  pub fn new(shape: IRect) -> Self {
    Panel {
      base: InodeBase::new(shape),
    }
  }
}

inode_impl!(Panel, base);

impl Widgetable for Panel {}
