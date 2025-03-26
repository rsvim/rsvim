//! Window root container.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

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

inode_impl!(WindowRootContainer, base);

impl Widgetable for WindowRootContainer {}
