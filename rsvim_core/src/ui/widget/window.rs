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

inode_itree_impl!(Window, base);

impl Widgetable for Window {
  fn draw(&self, canvas: &mut Canvas) {
    for node in self.base.iter() {
      // trace!("Draw window:{:?}", node);
      node.draw(canvas);
    }
  }
}

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

// Cursor {
impl Window {
  /// Enable/insert cursor widget in window, i.e. when user moves cursor to a window, the window
  /// content widget contains this cursor, and allow user moving cursor (or inserting text at
  /// cursor).
  ///
  /// # Returns
  /// It returns the old cursor widget if there's any, otherwise it returns `None`.
  pub fn insert_cursor(&mut self, cursor: Cursor) -> Option<WindowNode> {
    self.cursor_id = Some(cursor.id());
    self
      .base
      .bounded_insert(self.content_id, WindowNode::Cursor(cursor))
  }

  /// Disable/remove cursor widget from window, i.e. when user cursor leaves window, the window
  /// content widget doesn't contain this cursor any longer.
  ///
  /// # Returns
  /// It returns the removed cursor widget if exists, otherwise it returns `None`.
  pub fn remove_cursor(&mut self) -> Option<WindowNode> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node(cursor_id).is_some());
        debug_assert!(self.base.parent_id(cursor_id).is_some());
        debug_assert_eq!(
          self.base.parent_id(cursor_id).unwrap(),
          self.content_id
        );
        self.cursor_id = None;
        let cursor_node = self.base.move_child(cursor_id);
        debug_assert!(cursor_node.is_some());
        debug_assert!(matches!(
          cursor_node.as_ref().unwrap(),
          WindowNode::Cursor(_)
        ));
        cursor_node
      }
      None => {
        debug_assert!(self.cursor_id.is_none());
        debug_assert!(self.base.node(self.content_id).is_some());
        debug_assert!(self.base.children_ids(self.content_id).is_empty());
        None
      }
    }
  }

  /// Bounded move cursor by x(columns) and y(rows).
  ///
  /// # Panics
  /// It panics if cursor not exist.
  pub fn move_cursor_by(&mut self, x: isize, y: isize) -> Option<IRect> {
    let cursor_id = self.cursor_id.unwrap();
    self.base.bounded_move_by(cursor_id, x, y)
  }

  /// Bounded move cursor to position x(columns) and y(rows).
  ///
  /// # Panics
  /// It panics if cursor not exist.
  pub fn move_cursor_to(&mut self, x: isize, y: isize) -> Option<IRect> {
    let cursor_id = self.cursor_id.unwrap();
    self.base.reserved_move_position_to(cursor_id, x, y)
  }
}
// Cursor }
