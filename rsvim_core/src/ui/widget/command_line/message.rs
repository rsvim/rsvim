//! Commandline's message widget.

use crate::content::TextContentsWk;
use crate::inode_impl;
use crate::lock;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Commandline message.
pub struct CommandLineMessage {
  base: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

inode_impl!(CommandLineMessage);

impl CommandLineMessage {
  pub fn new(
    lotree: ItreeWk,
    id: TreeNodeId,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    CommandLineMessage {
      base: InodeBase::new(lotree, id),
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

impl Widgetable for CommandLineMessage {
  fn draw(&self, canvas: &mut Canvas) {
    if self.visible() {
      let actual_shape = self.actual_shape();
      let contents = self.text_contents.upgrade().unwrap();
      let contents = lock!(contents);
      let viewport = self.viewport.upgrade().unwrap();
      viewport.draw(contents.command_line_message(), &actual_shape, canvas);
    }
  }
}
