//! Vim window's text content widget.

use crate::buf::BufferWk;
use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use taffy::Style;
use taffy::TaffyResult;
use taffy::prelude::TaffyMaxContent;

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
    relationship: &mut Irelationship,
    style: Style,
    parent_id: TreeNodeId,
    buffer: BufferWk,
    viewport: ViewportWk,
  ) -> TaffyResult<Self> {
    let (content_id, content_shape) = {
      let rel = relationship;
      let content_id = rel.new_leaf(style)?;
      rel.add_child(parent_id, content_id)?;
      rel.compute_layout(content_id, taffy::Size::MAX_CONTENT)?;
      let content_layout = rel.layout(content_id)?;
      let content_shape = u16rect_from_layout!(content_layout);
      (content_id, content_shape)
    };

    let base = InodeBase::new(content_id, content_shape);
    Content {
      base,
      buffer,
      viewport,
    }
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
