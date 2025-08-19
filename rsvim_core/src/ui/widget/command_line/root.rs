//! Commandline root container.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone, Copy)]
/// Commandline root container.
pub struct RootContainer {
  base: InodeBase,
}

impl RootContainer {
  pub fn new(shape: IRect) -> Self {
    RootContainer {
      base: InodeBase::new(shape),
    }
  }
}

inode_impl!(RootContainer, base);

impl Widgetable for RootContainer {}
