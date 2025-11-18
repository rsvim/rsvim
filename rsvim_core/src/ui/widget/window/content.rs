//! Window content.

use crate::buf::BufferWk;
use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use taffy::Style;
use taffy::TaffyResult;

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct Content {
  base: InodeBase,
  buffer: BufferWk,
  viewport: ViewportWk,
}

impl Content {
  /// Make window content.
  pub fn new(
    lotree: TaffyTreeWk,
    style: Style,
    buffer: BufferWk,
    viewport: ViewportWk,
  ) -> TaffyResult<Self> {
    let base = InodeBase::new(lotree, style)?;
    Ok(Content {
      base,
      buffer,
      viewport,
    })
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

inode_impl!(Content, base);

impl Widgetable for Content {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let buffer = self.buffer.upgrade().unwrap();
    let buffer = lock!(buffer);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(buffer.text(), actual_shape, canvas);
  }
}
