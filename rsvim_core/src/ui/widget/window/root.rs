//! Window root container.

use crate::inode_generate_impl;
use crate::prelude::*;
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
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

inode_generate_impl!(WindowRootContainer, base);

impl Widgetable for WindowRootContainer {}
