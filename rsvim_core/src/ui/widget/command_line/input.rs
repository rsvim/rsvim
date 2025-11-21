//! Commandline's input content widget.

use crate::content::TextContentsWk;
use crate::inode_impl;
use crate::lock;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Commandline input content.
pub struct CommandLineInput {
  base: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

impl CommandLineInput {
  pub fn new(
    relationship: IrelationshipRc,
    id: TreeNodeId,
    shape: U16Rect,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    let base = InodeBase::new(relationship, id, shape);
    CommandLineInput {
      base,
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

inode_impl!(CommandLineInput, base);

impl Widgetable for CommandLineInput {
  fn draw(&self, canvas: &mut Canvas) {
    if self.base.visible() {
      let actual_shape = self.actual_shape();
      let contents = self.text_contents.upgrade().unwrap();
      let contents = lock!(contents);
      let viewport = self.viewport.upgrade().unwrap();
      viewport.draw(contents.command_line_input(), actual_shape, canvas);
    }
  }
}
