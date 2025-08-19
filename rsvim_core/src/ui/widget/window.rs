//! Window.

use crate::buf::BufferWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc,
};
use crate::ui::widget::Widgetable;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::content::Content;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::{inode_enum_dispatcher, inode_itree_impl, widget_enum_dispatcher};

// Re-export
pub use opt::*;

use std::sync::Arc;

pub mod content;
pub mod opt;
pub mod root;

#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod opt_tests;

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(Content),
  Cursor(Cursor),
}

inode_enum_dispatcher!(WindowNode, WindowRootContainer, WindowContent, Cursor);
widget_enum_dispatcher!(WindowNode, WindowRootContainer, WindowContent, Cursor);

#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
pub struct Window {
  base: Itree<WindowNode>,
  options: WindowLocalOptions,

  content_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,

  buffer: BufferWk,
  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

impl Window {
  pub fn new(
    opts: &WindowLocalOptions,
    shape: IRect,
    buffer: BufferWk,
  ) -> Self {
    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);
    let window_root_actual_shape = window_root.actual_shape();

    let mut base = Itree::new(window_root_node);

    let (viewport, cursor_viewport) = {
      let buffer = buffer.upgrade().unwrap();
      let buffer = lock!(buffer);
      let viewport =
        Viewport::view(opts, buffer.text(), window_root_actual_shape, 0, 0);
      let cursor_viewport =
        CursorViewport::from_top_left(&viewport, buffer.text());
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    let window_content =
      Content::new(shape, buffer.clone(), Arc::downgrade(&viewport));
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(window_root_id, window_content_node);

    Window {
      base,
      options: *opts,
      content_id: window_content_id,
      cursor_id: None,
      buffer,
      viewport,
      cursor_viewport,
    }
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
  pub fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  /// Set window local options.
  pub fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = *options;
  }

  /// Get viewport.
  pub fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Set viewport.
  pub fn set_viewport(&mut self, viewport: ViewportArc) {
    self.viewport = viewport.clone();
    if let Some(WindowNode::WindowContent(content)) =
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
  /// Window content widget.
  pub fn content(&self) -> &Content {
    debug_assert!(self.base.node(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.content_id).unwrap(),
      WindowNode::WindowContent(_)
    ));
    match self.base.node(self.content_id).unwrap() {
      WindowNode::WindowContent(w) => {
        debug_assert_eq!(w.id(), self.content_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable window content widget.
  pub fn content_mut(&mut self) -> &mut Content {
    debug_assert!(self.base.node_mut(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.content_id).unwrap(),
      WindowNode::WindowContent(_)
    ));
    match self.base.node_mut(self.content_id).unwrap() {
      WindowNode::WindowContent(w) => {
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
// Viewport }

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
