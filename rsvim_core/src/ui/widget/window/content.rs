//! Vim window's text content widget.

use crate::buf::BufferWk;
use crate::inodify;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct WindowContent {
  __node: InodeBase,
  buffer: BufferWk,
  viewport: ViewportWk,
}

inodify!(WindowContent);

impl WindowContent {
  /// Make window content.
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    buffer: BufferWk,
    viewport: ViewportWk,
  ) -> Self {
    WindowContent {
      __node: InodeBase::new(id, ctx),
      buffer,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

impl Widgetable for WindowContent {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let buffer = self.buffer.upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(buffer.text(), &actual_shape, canvas);
  }
}
