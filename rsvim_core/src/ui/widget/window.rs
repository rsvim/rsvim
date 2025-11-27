//! Window.

pub mod content;
pub mod opt;

#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod opt_tests;

use crate::buf::BufferWk;
use crate::inode_impl;
use crate::prelude::*;
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
use taffy::TaffyResult;

#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: InodeBase,
  options: WindowOptions,

  content_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,

  buffer: BufferWk,
  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

inode_impl!(Window);

impl Window {
  pub fn new(
    relationship: ItreeRc,
    id: TreeNodeId,
    opts: WindowOptions,
    content_id: TreeNodeId,
    buffer: BufferWk,
  ) -> TaffyResult<Self> {
    let (viewport, cursor_viewport) = {
      let base = relationship.borrow();
      let content_actual_shape = base.actual_shape(content_id)?;
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport =
        Viewport::view(&opts, buffer.text(), &content_actual_shape, 0, 0);
      let cursor_viewport =
        CursorViewport::from_top_left(&viewport, buffer.text());
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    let base = InodeBase::new(relationship, id);
    Ok(Window {
      base,
      options: opts,
      content_id,
      cursor_id: None,
      buffer,
      viewport,
      cursor_viewport,
    })
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

  /// Cursor widget ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
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
  pub fn set_viewport(&mut self, viewport: ViewportArc) {
    self.viewport = viewport.clone();
    if let Some(WindowNode::Content(content)) =
      self.base.node_mut(self.content_id)
    {
      content.set_viewport(Arc::downgrade(&viewport));
    }
  }

  /// Get cursor viewport.
  pub fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  pub fn set_cursor_viewport(&mut self, cursor_viewport: CursorViewportArc) {
    self.cursor_viewport = cursor_viewport;
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

// Sub-Widgets {
impl Window {
  /// Window content widget.
  pub fn content(&self) -> &WindowContent {
    debug_assert!(self.base.node(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.content_id).unwrap(),
      WindowNode::Content(_)
    ));
    match self.base.node(self.content_id).unwrap() {
      WindowNode::Content(w) => {
        debug_assert_eq!(w.id(), self.content_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable window content widget.
  pub fn content_mut(&mut self) -> &mut WindowContent {
    debug_assert!(self.base.node_mut(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.content_id).unwrap(),
      WindowNode::Content(_)
    ));
    match self.base.node_mut(self.content_id).unwrap() {
      WindowNode::Content(w) => {
        debug_assert_eq!(w.id(), self.content_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Cursor widget.
  pub fn cursor(&self) -> Option<&Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node(cursor_id).is_some());
        debug_assert!(matches!(
          self.base.node(cursor_id).unwrap(),
          WindowNode::Cursor(_)
        ));
        match self.base.node(cursor_id).unwrap() {
          WindowNode::Cursor(c) => {
            debug_assert_eq!(c.id(), cursor_id);
            Some(c)
          }
          _ => None,
        }
      }
      None => None,
    }
  }

  pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node_mut(cursor_id).is_some());
        debug_assert!(matches!(
          self.base.node_mut(cursor_id).unwrap(),
          WindowNode::Cursor(_)
        ));
        match self.base.node_mut(cursor_id).unwrap() {
          WindowNode::Cursor(c) => {
            debug_assert_eq!(c.id(), cursor_id);
            Some(c)
          }
          _ => None,
        }
      }
      None => None,
    }
  }
}
// Sub-Widgets }

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
        let cursor_node = self.base.remove(cursor_id);
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
    self.base.bounded_move_to(cursor_id, x, y)
  }
}
// Cursor }
