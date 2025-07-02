//! Commandline root container.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone, Copy)]
/// Commandline root container.
pub struct CommandLineRootContainer {
  base: InodeBase,
}

impl CommandLineRootContainer {
  pub fn new(shape: IRect) -> Self {
    CommandLineRootContainer {
      base: InodeBase::new(shape),
    }
  }
}

inode_impl!(CommandLineRootContainer, base);

impl Widgetable for CommandLineRootContainer {}
