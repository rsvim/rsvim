//! Command-line widget.

use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Viewportable};
use crate::ui::widget::Widgetable;
use crate::ui::widget::command_line::content::CommandLineContent;
use crate::ui::widget::command_line::indicator::CommandLineIndicator;
use crate::ui::widget::command_line::root::CommandLineRootContainer;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::opt::{WindowLocalOptions, WindowLocalOptionsBuilder};
use crate::{inode_enum_dispatcher, inode_impl, widget_enum_dispatcher};

pub mod content;
pub mod indicator;
pub mod root;

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

  contents: TextContentsWk,

  viewport: ViewportArc,
  cursor_viewport: CursorViewportArc,
}

impl CommandLine {
  pub fn new(shape: IRect, contents: TextContentsWk) -> Self {
    // Force cmdline window options.
    let options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0_u16)
      .build()
      .unwrap();

    let cmdline_root = CommandLineRootContainer::new(shape);
    let cmdline_root_id = cmdline_root.id();
    let cmdline_root_node = CommandLineNode::CommandLineRootContainer(cmdline_root);

    let cmdline_indicator = CommandLineIndicator::new(shape, buffer, viewport);

    let mut base = Itree::new(cmdline_root_node);
    let cmdline_actual_shape = base.actual_shape();

    let (viewport, cursor_viewport) = {
      let contents = contents.upgrade().unwrap();
      let contents = lock!(contents);
      let viewport = Viewport::view(
        &options,
        contents.command_line_content(),
        cmdline_actual_shape,
        0,
        0,
      );
      let cursor_viewport =
        CursorViewport::from_top_left(&viewport, contents.command_line_content());
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    Self {
      base,
      options,
      contents,
      viewport,
      cursor_viewport,
    }
  }
}

inode_impl!(CommandLine, base);

impl Widgetable for CommandLine {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.clone();

    viewport.draw(contents.command_line_content(), actual_shape, canvas);
  }
}

impl Viewportable for CommandLine {
  /// Get window local options.
  fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  /// Set window local options.
  fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = *options;
  }

  /// Get viewport.
  fn viewport(&self) -> ViewportArc {
    self.viewport.clone()
  }

  /// Set viewport.
  fn set_viewport(&mut self, viewport: ViewportArc) {
    self.viewport = viewport;
  }

  /// Get cursor viewport.
  fn cursor_viewport(&self) -> CursorViewportArc {
    self.cursor_viewport.clone()
  }

  /// Set cursor viewport.
  fn set_cursor_viewport(&mut self, cursor_viewport: CursorViewportArc) {
    self.cursor_viewport = cursor_viewport;
  }
}

// Attributes {
impl CommandLine {
  /// Get cursor widget ID.
  ///
  /// # Returns
  /// It returns widget node ID if cursor is inside command-line, otherwise returns `None`.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
  }

  /// Get command-line content widget ID.
  pub fn content_id(&self) -> TreeNodeId {
    self.content_id
  }

  /// Get global text contents.
  pub fn text_contents(&self) -> TextContentsWk {
    self.contents.clone()
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
    let parent_id = self.content_id;
    self
      .base
      .bounded_insert(parent_id, CommandLineNode::Cursor(cursor))
  }

  /// Disable/remove cursor widget from commandline, i.e. when user leaves command-line mode, the
  /// cursor leaves the command-line widget and command-line widget doesn't contain cursor any
  /// longer.
  ///
  /// # Returns
  /// It returns the removed cursor widget if exists, otherwise it returns `None`.
  pub fn remove_cursor(&mut self) -> Option<CommandLineNode> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.base.node(cursor_id).is_some());
        debug_assert!(self.base.parent_id(cursor_id).is_some());
        debug_assert_eq!(self.base.parent_id(cursor_id).unwrap(), self.content_id);
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
        debug_assert!(self.cursor_id().is_none());
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
