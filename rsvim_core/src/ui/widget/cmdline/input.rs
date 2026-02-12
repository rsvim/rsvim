//! Commandline's input content widget.

use crate::cmdltext::CmdlineTextWk;
use crate::inodify_impl;
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
  cmdline_text: CmdlineTextWk,
  viewport: ViewportWk,
}

inodify_impl!(CmdlineInput);

impl CmdlineInput {
  pub fn new(
    id: NodeId,
    ctx: TreeContextWk,
    cmdline_text: CmdlineTextWk,
    viewport: ViewportWk,
  ) -> Self {
    CmdlineInput {
      __node: InodeBase::new(id, ctx),
      cmdline_text,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

impl Widgetable for CmdlineInput {
  fn draw(&self, canvas: &mut Canvas) {
    if self.enabled() {
      let actual_shape = self.actual_shape();
      let cmdline_text = self.cmdline_text.upgrade().unwrap();
      let cmdline_text = lock!(cmdline_text);
      let viewport = self.viewport.upgrade().unwrap();

      viewport.draw(cmdline_text.input(), &actual_shape, canvas);
    }
  }
}
