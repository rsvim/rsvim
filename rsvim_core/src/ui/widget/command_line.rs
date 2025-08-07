//! Command-line widget.

use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{CursorViewportArc, ViewportArc};
use crate::ui::widget::Widgetable;
use crate::ui::widget::command_line::content::CommandLineContent;
use crate::ui::widget::command_line::indicator::CommandLineIndicator;
use crate::ui::widget::command_line::root::CommandLineRootContainer;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::opt::WindowLocalOptions;
use crate::{inode_enum_dispatcher, inode_itree_impl, widget_enum_dispatcher};

use std::sync::Arc;

// Re-export
pub use indicator::CommandLineIndicatorSymbol;

pub mod builder;
pub mod content;
pub mod indicator;
pub mod root;

#[cfg(test)]
pub mod indicator_tests;

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum CommandLineNode {
  CommandLineRootContainer(CommandLineRootContainer),
  CommandLineIndicator(CommandLineIndicator),
  CommandLineContent(CommandLineContent),
  Cursor(Cursor),
}

inode_enum_dispatcher!(
  CommandLineNode,
  CommandLineRootContainer,
  CommandLineIndicator,
  CommandLineContent,
  Cursor
);
widget_enum_dispatcher!(
  CommandLineNode,
  CommandLineRootContainer,
  CommandLineIndicator,
  CommandLineContent,
  Cursor
);

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: Itree<CommandLineNode>,
  options: WindowLocalOptions,

  indicator_id: TreeNodeId,
  content_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,

  text_contents: TextContentsWk,

  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

inode_itree_impl!(CommandLine, base);

impl Widgetable for CommandLine {
  fn draw(&self, canvas: &mut Canvas) {
    for node in self.base.iter() {
      // trace!("Draw window:{:?}", node);
      node.draw(canvas);
    }
  }
}

impl CommandLine {
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
    if let Some(CommandLineNode::CommandLineContent(content)) =
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

  /// Get binded global text contents.
  pub fn text_contents(&self) -> TextContentsWk {
    self.text_contents.clone()
  }

  /// Cursor widget ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
  }

  /// Command-line indicator widget ID.
  pub fn indicator_id(&self) -> TreeNodeId {
    self.indicator_id
  }

  /// Command-line content widget ID.
  pub fn content_id(&self) -> TreeNodeId {
    self.content_id
  }
}

// Widgets {
impl CommandLine {
  /// Command-line content widget.
  pub fn content(&self) -> &CommandLineContent {
    debug_assert!(self.base.node(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.content_id).unwrap(),
      CommandLineNode::CommandLineContent(_)
    ));

    match self.base.node(self.content_id).unwrap() {
      CommandLineNode::CommandLineContent(w) => {
        debug_assert_eq!(w.id(), self.content_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable command-line content widget.
  pub fn content_mut(&mut self) -> &mut CommandLineContent {
    debug_assert!(self.base.node_mut(self.content_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.content_id).unwrap(),
      CommandLineNode::CommandLineContent(_)
    ));

    match self.base.node_mut(self.content_id).unwrap() {
      CommandLineNode::CommandLineContent(w) => {
        debug_assert_eq!(w.id(), self.content_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Command-line indicator widget.
  pub fn indicator(&self) -> &CommandLineIndicator {
    debug_assert!(self.base.node(self.indicator_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.indicator_id).unwrap(),
      CommandLineNode::CommandLineIndicator(_)
    ));

    match self.base.node(self.indicator_id).unwrap() {
      CommandLineNode::CommandLineIndicator(w) => {
        debug_assert_eq!(w.id(), self.indicator_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable command-line indicator widget.
  pub fn indicator_mut(&mut self) -> &mut CommandLineIndicator {
    debug_assert!(self.base.node_mut(self.indicator_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.indicator_id).unwrap(),
      CommandLineNode::CommandLineIndicator(_)
    ));

    match self.base.node_mut(self.indicator_id).unwrap() {
      CommandLineNode::CommandLineIndicator(w) => {
        debug_assert_eq!(w.id(), self.indicator_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Command-line cursor widget.
  pub fn cursor(&self) -> Option<&Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node(cursor_id).is_some());
        debug_assert!(matches!(
          self.base.node(cursor_id).unwrap(),
          CommandLineNode::Cursor(_)
        ));

        match self.base.node(cursor_id).unwrap() {
          CommandLineNode::Cursor(w) => {
            debug_assert_eq!(w.id(), cursor_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
      None => None,
    }
  }

  /// Mutable command-line cursor widget.
  pub fn cursor_mut(&mut self) -> Option<&mut Cursor> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node_mut(cursor_id).is_some());
        debug_assert!(matches!(
          self.base.node_mut(cursor_id).unwrap(),
          CommandLineNode::Cursor(_)
        ));

        match self.base.node_mut(cursor_id).unwrap() {
          CommandLineNode::Cursor(w) => {
            debug_assert_eq!(w.id(), cursor_id);
            Some(w)
          }
          _ => unreachable!(),
        }
      }
      None => None,
    }
  }
}
// Attributes }

// Cursor {
impl CommandLine {
  /// Enable/insert cursor widget in commandline, i.e. when user start command-line mode, the
  /// cursor moves to the command-line widget and allow receive user ex command or search patterns.
  ///
  /// # Returns
  /// It returns the old cursor widget if there's any, otherwise it returns `None`.
  pub fn insert_cursor(&mut self, cursor: Cursor) -> Option<CommandLineNode> {
    self.cursor_id = Some(cursor.id());
    self
      .base
      .bounded_insert(self.content_id, CommandLineNode::Cursor(cursor))
  }

  /// Disable/remove cursor widget from commandline, i.e. when user leaves command-line mode, the
  /// command-line content widget doesn't contain cursor any longer.
  ///
  /// # Returns
  /// It returns the removed cursor widget if exists, otherwise it returns `None`.
  pub fn remove_cursor(&mut self) -> Option<CommandLineNode> {
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
          CommandLineNode::Cursor(_)
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
