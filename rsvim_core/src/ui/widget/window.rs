//! Window.

pub mod content;
pub mod opt;

#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod opt_tests;

use crate::buf::BufferWk;
use crate::inode_itree_impl;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::CursorViewportArc;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::EditableWidgetable;
use crate::ui::widget::Widgetable;
use crate::ui::widget::cursor::Cursor;
use content::WindowContent;
use opt::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Window {
  base: InodeBase,
  options: WindowOptions,

  content_id: TreeNodeId,

  buffer: BufferWk,
  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

impl Window {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    options: WindowOptions,
    buffer: BufferWk,
    content_id: TreeNodeId,
    content_size: &U16Size,
  ) -> Self {
    let base = InodeBase::new(id, ctx);

    let (viewport, cursor_viewport) = {
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport =
        Viewport::view(&options, buffer.text(), content_size, 0, 0);
      let cursor_viewport =
        CursorViewport::from_top_left(&viewport, buffer.text());
      (
        Viewport::to_arc(viewport),
        CursorViewport::to_arc(cursor_viewport),
      )
    };

    Window {
      base,
      options,
      content_id,
      buffer,
      viewport,
      cursor_viewport,
    }
  }

  /// This is only for setting window content id after constructor.
  pub fn __post_initialize_content_id(&mut self, value: TreeNodeId) {
    self.content_id = value;
  }
}

impl Widgetable for Window {}

// Attributes
impl Window {
  /// Get window local options.
  pub fn options(&self) -> &WindowOptions {
    &self.options
  }

  /// Set window local options.
  pub fn set_options(&mut self, options: &WindowOptions) {
    self.options = *options;
  }

  /// Get binded buffer.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  /// Content widget ID.
  pub fn content_id(&self) -> TreeNodeId {
    self.content_id
  }
}

// Viewport {
impl Window {
  /// Get viewport.
  pub fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Set viewport.
  pub fn set_viewport(&mut self, viewport: Viewport) {
    *lock!(self.viewport) = viewport;
  }

  /// Get cursor viewport.
  pub fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  pub fn set_cursor_viewport(&mut self, cursor_viewport: CursorViewport) {
    *lock!(self.cursor_viewport) = cursor_viewport;
  }
}
// Viewport }

// Editable Viewport {
impl EditableWidgetable for Window {
  fn editable_viewport(&self) -> ViewportArc {
    self.viewport()
  }

  fn set_editable_viewport(&mut self, viewport: ViewportArc) {
    self.set_viewport(viewport);
  }

  fn editable_cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport()
  }

  fn set_editable_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  ) {
    self.set_cursor_viewport(cursor_viewport);
  }

  fn editable_options(&self) -> &WindowOptions {
    self.options()
  }

  fn editable_actual_shape(&self) -> &U16Rect {
    self.content().actual_shape()
  }

  fn move_editable_cursor_to(&mut self, x: isize, y: isize) -> Option<IRect> {
    self.move_cursor_to(x, y)
  }

  fn editable_cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id()
  }
}
// Editable Viewport }
