//! Commandline's message widget.

use crate::cmdltext::CmdlineTextWk;
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
  cmdline_text: CmdlineTextWk,
  viewport: ViewportWk,
}

inodify_impl!(CmdlineMessage);

impl CmdlineMessage {
  pub fn new(
    id: NodeId,
    ctx: TreeContextWk,
    cmdline_text: CmdlineTextWk,
    viewport: ViewportWk,
  ) -> Self {
    CmdlineMessage {
      __node: InodeBase::new(id, ctx),
      cmdline_text,
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
      let cmdline_text = self.cmdline_text.upgrade().unwrap();
      let cmdline_text = lock!(cmdline_text);
      let viewport = self.viewport.upgrade().unwrap();

      viewport.draw(cmdline_text.cmdline_message(), &actual_shape, canvas);
    }
  }
}
