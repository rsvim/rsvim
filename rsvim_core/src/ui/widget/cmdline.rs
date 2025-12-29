//! Command-line widget.

pub mod indicator;
pub mod input;
pub mod message;

#[cfg(test)]
pub mod indicator_tests;

use crate::content::TextContentsWk;
use crate::inode_dispatcher;
use crate::inode_impl;
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
use crate::ui::widget::panel::Panel;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_dispatcher;
use indicator::CmdlineIndicator;
use indicator::CmdlineIndicatorSymbol;
use input::CmdlineInput;
use message::CmdlineMessage;
use std::sync::Arc;

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct Cmdline {
  __node: InodeBase,
  options: WindowOptions,

  input_panel_id: TreeNodeId,
  indicator_id: TreeNodeId,
  input_id: TreeNodeId,
  message_id: TreeNodeId,

  input_viewport: ViewportArc,
  input_cursor_viewport: CursorViewportArc,
  message_viewport: ViewportArc,
}

inode_impl!(Cmdline);

impl Cmdline {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    text_contents: TextContentsWk,
    input_panel_id: TreeNodeId,
    indicator_id: TreeNodeId,
    input_id: TreeNodeId,
    input_size: &U16Size,
    message_id: TreeNodeId,
    message_size: &U16Size,
  ) -> Self {
    // Force cmdline window options.
    let options = WindowOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0)
      .build()
      .unwrap();

    let (input_viewport, input_cursor_viewport, message_viewport) = {
      let text_contents = text_contents.upgrade().unwrap();
      let text_contents = lock!(text_contents);
      let input_viewport = Viewport::view(
        &options,
        text_contents.command_line_input(),
        input_size,
        0,
        0,
      );
      let input_cursor_viewport = CursorViewport::from_top_left(
        &input_viewport,
        text_contents.command_line_input(),
      );

      let message_viewport = Viewport::view(
        &options,
        text_contents.command_line_message(),
        message_size,
        0,
        0,
      );
      (input_viewport, input_cursor_viewport, message_viewport)
    };

    let input_viewport = Viewport::to_arc(input_viewport);
    let input_cursor_viewport = CursorViewport::to_arc(input_cursor_viewport);
    let message_viewport = Viewport::to_arc(message_viewport);

    Self {
      __node: InodeBase::new(id, ctx),
      options,
      input_panel_id,
      indicator_id,
      input_id,
      message_id,
      input_viewport,
      input_cursor_viewport,
      message_viewport,
    }
  }
}

impl Widgetable for Cmdline {}

impl Cmdline {
  /// Get window local options.
  pub fn options(&self) -> &WindowOptions {
    &self.options
  }

  /// Set window local options.
  pub fn set_options(&mut self, options: &WindowOptions) {
    self.options = *options;
  }

  /// Command-line input panel widget ID.
  pub fn input_panel_id(&self) -> TreeNodeId {
    self.input_panel_id
  }

  /// Command-line indicator widget ID.
  pub fn indicator_id(&self) -> TreeNodeId {
    self.indicator_id
  }

  /// Command-line input widget ID.
  pub fn input_id(&self) -> TreeNodeId {
    self.input_id
  }

  /// Command-line message widget ID.
  pub fn message_id(&self) -> TreeNodeId {
    self.message_id
  }
}

// Viewport {
impl Cmdline {
  /// Get input viewport.
  pub fn input_viewport(&self) -> ViewportArc {
    self.input_viewport.clone()
  }

  /// Get message viewport.
  pub fn message_viewport(&self) -> ViewportArc {
    self.message_viewport.clone()
  }

  /// Set viewport for input.
  pub fn set_input_viewport(&mut self, viewport: Viewport) {
    *lock!(self.input_viewport) = viewport;
  }

  /// Set viewport for message.
  pub fn set_message_viewport(&mut self, viewport: Viewport) {
    *lock!(self.message_viewport) = viewport;
  }

  /// Get cursor viewport for input.
  pub fn input_cursor_viewport(&self) -> CursorViewportArc {
    self.input_cursor_viewport.clone()
  }

  /// Set cursor viewport for input.
  pub fn set_input_cursor_viewport(&mut self, cursor_viewport: CursorViewport) {
    *lock!(self.input_cursor_viewport) = cursor_viewport;
  }
}
// Viewport }

// Editable Viewport {
impl EditableWidgetable for Cmdline {
  fn editable_viewport(&self) -> ViewportArc {
    self.input_viewport()
  }

  fn set_editable_viewport(&mut self, viewport: ViewportArc) {
    self.set_input_viewport(viewport);
  }

  fn editable_cursor_viewport(&self) -> CursorViewportArc {
    self.input_cursor_viewport()
  }

  fn set_editable_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  ) {
    self.set_input_cursor_viewport(cursor_viewport);
  }

  fn editable_options(&self) -> &WindowOptions {
    self.options()
  }

  fn editable_actual_shape(&self) -> &U16Rect {
    self.input().actual_shape()
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
impl Cmdline {
  /// Enable/insert cursor widget in commandline, i.e. when user start command-line mode, the
  /// cursor moves to the command-line widget and allow receive user ex command or search patterns.
  ///
  /// # Returns
  /// It returns the old cursor widget if there's any, otherwise it returns `None`.
  pub fn insert_cursor(&mut self, cursor: Cursor) -> Option<CommandLineNode> {
    self.cursor_id = Some(cursor.id());
    self
      .__node
      .bounded_insert(self.input_id, CommandLineNode::Cursor(cursor))
  }

  /// Disable/remove cursor widget from commandline, i.e. when user leaves command-line mode, the
  /// command-line content widget doesn't contain cursor any longer.
  ///
  /// # Returns
  /// It returns the removed cursor widget if exists, otherwise it returns `None`.
  pub fn remove_cursor(&mut self) -> Option<CommandLineNode> {
    match self.cursor_id {
      Some(cursor_id) => {
        debug_assert!(self.__node.node(cursor_id).is_some());
        debug_assert!(self.__node.parent_id(cursor_id).is_some());
        debug_assert_eq!(
          self.__node.parent_id(cursor_id).unwrap(),
          self.input_id
        );
        self.cursor_id = None;
        let cursor_node = self.__node.move_child(cursor_id);
        debug_assert!(cursor_node.is_some());
        debug_assert!(matches!(
          cursor_node.as_ref().unwrap(),
          CommandLineNode::Cursor(_)
        ));
        cursor_node
      }
      None => {
        debug_assert!(self.cursor_id.is_none());
        debug_assert!(self.__node.node(self.input_id).is_some());
        debug_assert!(self.__node.children_ids(self.input_id).is_empty());
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
    self.__node.bounded_move_by(cursor_id, x, y)
  }

  /// Bounded move cursor to position x(columns) and y(rows).
  ///
  /// # Panics
  /// It panics if cursor not exist.
  pub fn move_cursor_to(&mut self, x: isize, y: isize) -> Option<IRect> {
    let cursor_id = self.cursor_id.unwrap();
    self.__node.reserved_move_position_to(cursor_id, x, y)
  }
}
// Cursor }
