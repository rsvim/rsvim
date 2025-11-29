//! Command-line widget.

pub mod indicator;
pub mod input;
pub mod message;

use crate::content::TextContentsWk;
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
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use indicator::CommandLineIndicator;
use input::CommandLineInput;
use message::CommandLineMessage;
use std::sync::Arc;
use taffy::TaffyResult;

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: InodeBase,
  options: WindowOptions,

  indicator_id: TreeNodeId,
  input_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,
  message_id: TreeNodeId,

  input_viewport: ViewportArc,
  input_cursor_viewport: CursorViewportArc,
  message_viewport: ViewportArc,
}

inode_impl!(CommandLine);
impl Widgetable for CommandLine {}

impl CommandLine {
  pub fn new(
    lotree: ItreeWk,
    id: TreeNodeId,
    indicator_id: TreeNodeId,
    input_id: TreeNodeId,
    message_id: TreeNodeId,
    text_contents: TextContentsWk,
  ) -> TaffyResult<Self> {
    // Force cmdline window options.
    let options = WindowOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0)
      .build()
      .unwrap();

    let (input_viewport, input_cursor_viewport, message_viewport) = {
      let lotree = lotree.upgrade().unwrap();
      let lotree = lotree.borrow();
      let input_actual_shape = lotree.actual_shape(input_id)?;
      let text_contents = text_contents.upgrade().unwrap();
      let text_contents = lock!(text_contents);
      let input_viewport = Viewport::view(
        &options,
        text_contents.command_line_input(),
        &input_actual_shape,
        0,
        0,
      );
      let input_cursor_viewport = CursorViewport::from_top_left(
        &input_viewport,
        text_contents.command_line_input(),
      );

      let message_actual_shape = lotree.actual_shape(message_id)?;
      let message_viewport = Viewport::view(
        &options,
        text_contents.command_line_message(),
        &message_actual_shape,
        0,
        0,
      );
      (input_viewport, input_cursor_viewport, message_viewport)
    };
    let input_viewport = Viewport::to_arc(input_viewport);
    let input_cursor_viewport = CursorViewport::to_arc(input_cursor_viewport);
    let message_viewport = Viewport::to_arc(message_viewport);

    Ok(Self {
      base: InodeBase::new(lotree, id),
      options,
      indicator_id,
      input_id,
      message_id,
      cursor_id: None,
      input_viewport,
      input_cursor_viewport,
      message_viewport,
    })
  }
}

impl CommandLine {
  /// Get window local options.
  pub fn options(&self) -> &WindowOptions {
    &self.options
  }

  /// Set window local options.
  pub fn set_options(&mut self, options: &WindowOptions) {
    self.options = *options;
  }

  /// Cursor widget ID.
  pub fn cursor_id(&self) -> Option<TreeNodeId> {
    self.cursor_id
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
impl CommandLine {
  /// Get input viewport.
  pub fn input_viewport(&self) -> ViewportArc {
    self.input_viewport.clone()
  }

  /// Set viewport for input.
  pub fn set_input_viewport(&mut self, viewport: ViewportArc) {
    self.input_viewport = viewport;
  }

  /// Get message viewport.
  pub fn message_viewport(&self) -> ViewportArc {
    self.message_viewport.clone()
  }

  /// Set viewport for message.
  pub fn set_message_viewport(&mut self, viewport: ViewportArc) {
    self.message_viewport = viewport;
  }

  /// Get cursor viewport for input.
  pub fn input_cursor_viewport(&self) -> CursorViewportArc {
    self.input_cursor_viewport.clone()
  }

  /// Set cursor viewport for input.
  pub fn set_input_cursor_viewport(&mut self, viewport: CursorViewportArc) {
    self.input_cursor_viewport = viewport;
  }
}
// Viewport }

// Editable Viewport {
impl EditableWidgetable for CommandLine {
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

// Show/Hide switch {
impl CommandLine {
  pub fn show_message(&mut self) {
    self.indicator_mut().set_visible(false);
    self.input_mut().set_visible(false);
    self.message_mut().set_visible(true);
  }

  pub fn show_input(&mut self) {
    self.indicator_mut().set_visible(true);
    self.input_mut().set_visible(true);
    self.message_mut().set_visible(false);
  }
}
// Show/Hide switch }

// Widgets {
impl CommandLine {
  /// Command-line input widget.
  pub fn input(&self) -> &CommandLineInput {
    debug_assert!(self.base.node(self.input_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.input_id).unwrap(),
      CommandLineNode::Input(_)
    ));

    match self.base.node(self.input_id).unwrap() {
      CommandLineNode::Input(w) => {
        debug_assert_eq!(w.id(), self.input_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable command-line input widget.
  pub fn input_mut(&mut self) -> &mut CommandLineInput {
    debug_assert!(self.base.node_mut(self.input_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.input_id).unwrap(),
      CommandLineNode::Input(_)
    ));

    match self.base.node_mut(self.input_id).unwrap() {
      CommandLineNode::Input(w) => {
        debug_assert_eq!(w.id(), self.input_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Command-line message widget
  pub fn message(&self) -> &CommandLineMessage {
    debug_assert!(self.base.node(self.message_id).is_some());
    debug_assert!(matches!(
      self.base.node(self.message_id).unwrap(),
      CommandLineNode::Message(_)
    ));

    match self.base.node(self.message_id).unwrap() {
      CommandLineNode::Message(w) => {
        debug_assert_eq!(w.id(), self.message_id);
        w
      }
      _ => unreachable!(),
    }
  }

  /// Mutable command-line message widget.
  pub fn message_mut(&mut self) -> &mut CommandLineMessage {
    debug_assert!(self.base.node_mut(self.message_id).is_some());
    debug_assert!(matches!(
      self.base.node_mut(self.message_id).unwrap(),
      CommandLineNode::Message(_)
    ));

    match self.base.node_mut(self.message_id).unwrap() {
      CommandLineNode::Message(w) => {
        debug_assert_eq!(w.id(), self.message_id);
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
      CommandLineNode::Indicator(_)
    ));

    match self.base.node(self.indicator_id).unwrap() {
      CommandLineNode::Indicator(w) => {
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
      CommandLineNode::Indicator(_)
    ));

    match self.base.node_mut(self.indicator_id).unwrap() {
      CommandLineNode::Indicator(w) => {
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
  /// Set cursor ID in commandline, e.g. user starts command-line mode, and
  /// cursor moves into the command-line widget.
  ///
  /// # Returns
  /// It returns the previous cursor ID.
  pub fn set_cursor_id(&mut self, cursor_id: TreeNodeId) -> Option<TreeNodeId> {
    let old = self.cursor_id;
    self.cursor_id = Some(cursor_id);
    old
  }

  /// Clear cursor widget from commandline, e.g. user leaves command-line mode,
  /// cursor moves out of command-line widget.
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
