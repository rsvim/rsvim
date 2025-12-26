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
pub struct CmdlineMessage {
  base: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

impl CmdlineMessage {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    let base = InodeBase::new(id, ctx);
    CmdlineMessage {
      base,
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

inode_impl!(CmdlineMessage, base);

impl Widgetable for CmdlineMessage {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.text_contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(contents.command_line_message(), actual_shape, canvas);
  }
}
