//! Window root container.

use crate::cart::{IRect, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::tree::internal::inode::{Inode, InodeBase, InodeId};
use crate::ui::widget::{Widget, WidgetId};

#[derive(Debug, Clone, Copy)]
/// Window root container.
pub struct WindowRootContainer {
  base: InodeBase,
}

impl WindowRootContainer {
  pub fn new(shape: IRect) -> Self {
    WindowRootContainer {
      base: InodeBase::new(shape),
    }
  }
}

inode_value_generate_impl!(WindowRootContainer, base);

impl Widget for WindowRootContainer {}
