//! Root container is the root node in the widget tree.

use crate::inodify_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Logical node that renders nothing but give a cerntain shape for its
/// descendant nodes.
pub struct Panel {
  __node: InodeBase,
}

inodify_impl!(Panel);

impl Panel {
  pub fn new(id: NodeId, ctx: TreeContextWk) -> Self {
    Panel {
      __node: InodeBase::new(id, ctx),
    }
  }
}

impl Widgetable for Panel {}
