//! Root container is the root node in the widget tree.

use crate::cart::{IRect, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::widget::{Widget, WidgetId};

#[derive(Debug, Clone, Copy)]
/// Root container.
pub struct RootContainer {
  base: Inode,
}

impl RootContainer {
  pub fn new(shape: IRect) -> Self {
    RootContainer {
      base: Inode::new(shape),
    }
  }
}

impl Widget for RootContainer {
  fn id(&self) -> WidgetId {
    self.base.id()
  }
}

inode_value_generate_impl!(RootContainer, base);
