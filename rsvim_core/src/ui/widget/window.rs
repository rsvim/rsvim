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
impl Widgetable for Window {}

impl Window {
  pub fn new(
    lotree: ItreeWk,
    id: TreeNodeId,
    opts: WindowOptions,
    content_id: TreeNodeId,
    buffer: BufferWk,
  ) -> TaffyResult<Self> {
    let (viewport, cursor_viewport) = {
      let lotree = lotree.upgrade().unwrap();
      let lotree = lotree.borrow();
      let content_actual_shape = lotree.shape(content_id)?;
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

    Ok(Window {
      base: InodeBase::new(lotree, id),
      options: opts,
      content_id,
      cursor_id: None,
      buffer,
      viewport,
      cursor_viewport,
    })
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
    self.viewport = viewport;
  }

  /// Get cursor viewport.
  pub fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  pub fn set_cursor_viewport(&mut self, viewport: CursorViewportArc) {
    self.cursor_viewport = viewport;
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
  /// Set cursor widget ID in window, e.g. user moves cursor into this window.
  ///
  /// # Returns
  /// It returns the previous cursor ID.
  pub fn set_cursor_id(&mut self, cursor_id: TreeNodeId) -> Option<TreeNodeId> {
    let old = self.cursor_id;
    self.cursor_id = Some(cursor_id);
    old
  }

  /// Clear cursor ID from window, e.g. user cursor leaves this window.
  ///
  /// # Returns
  /// It returns the previous cursor ID.
  pub fn clear_cursor_id(&mut self) -> Option<TreeNodeId> {
    let old = self.cursor_id;
    self.cursor_id = None;
    old
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
