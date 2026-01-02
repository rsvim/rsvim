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
pub struct CmdlineInput {
  __node: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

inode_impl!(CmdlineInput);

impl CmdlineInput {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    CmdlineInput {
      __node: InodeBase::new(id, ctx),
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

impl Widgetable for CmdlineInput {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.text_contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(contents.command_line_input(), &actual_shape, canvas);
  }
}
