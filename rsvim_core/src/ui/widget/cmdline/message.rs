//! Commandline's message widget.

use crate::content::TextContentsWk;
use crate::inodify_impl;
use crate::lock;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// Commandline message.
pub struct CmdlineMessage {
  __node: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

inodify_impl!(CmdlineMessage);

impl CmdlineMessage {
  pub fn new(
    id: NodeId,
    ctx: TreeContextWk,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    CmdlineMessage {
      __node: InodeBase::new(id, ctx),
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

impl Widgetable for CmdlineMessage {
  fn draw(&self, canvas: &mut Canvas) {
    if self.enabled() {
      let actual_shape = self.actual_shape();
      let contents = self.text_contents.upgrade().unwrap();
      let contents = lock!(contents);
      let viewport = self.viewport.upgrade().unwrap();

      viewport.draw(contents.cmdline_message(), &actual_shape, canvas);
    }
  }
}
