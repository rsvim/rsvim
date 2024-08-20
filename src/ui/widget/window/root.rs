//! Window root container.

use crate::cart::{IRect, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::widget::{Widget, WidgetId};

#[derive(Debug, Clone, Copy)]
/// Window root container.
pub struct WindowRootContainer {
  base: Inode,
}

impl WindowRootContainer {
  pub fn new(shape: IRect) -> Self {
    WindowRootContainer {
      base: Inode::new(shape),
    }
  }
}

impl Widget for WindowRootContainer {
  fn id(&self) -> WidgetId {
    self.base.id()
  }
}

inode_value_generate_impl!(WindowRootContainer, base);
