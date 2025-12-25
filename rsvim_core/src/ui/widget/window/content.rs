//! Vim window's text content widget.

use crate::buf::BufferWk;
use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct WindowContent {
  base: InodeBase,

  // Buffer.
  buffer: BufferWk,

  // Viewport.
  viewport: ViewportWk,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect, buffer: BufferWk, viewport: ViewportWk) -> Self {
    let base = InodeBase::new(shape);
    WindowContent {
      base,
      buffer,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

inode_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let buffer = self.buffer.upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(buffer.text(), actual_shape, canvas);
  }
}
