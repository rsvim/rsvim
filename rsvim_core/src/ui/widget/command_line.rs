//! Command-line widget.

use crate::content::TextContentsWk;
use crate::geo_rect_as;
use crate::inode_enum_dispatcher;
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
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use crate::widget_enum_dispatcher;
use indicator::Indicator;
use indicator::IndicatorSymbol;
use input::Input;
use message::Message;
use root::RootContainer;
use std::sync::Arc;

pub mod indicator;
pub mod input;
pub mod message;
pub mod root;

#[cfg(test)]
pub mod indicator_tests;

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum CommandLineNode {
  RootContainer(RootContainer),
  Indicator(Indicator),
  Input(Input),
  Cursor(Cursor),
  Message(Message),
}

inode_enum_dispatcher!(
  CommandLineNode,
  RootContainer,
  Indicator,
  Input,
  Cursor,
  Message
);

widget_enum_dispatcher!(
  CommandLineNode,
  RootContainer,
  Indicator,
  Input,
  Cursor,
  Message
);

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: Itree<CommandLineNode>,
  options: WindowOptions,

  indicator_id: TreeNodeId,
  input_id: TreeNodeId,
  cursor_id: Option<TreeNodeId>,
  message_id: TreeNodeId,

  input_viewport: ViewportArc,
  input_cursor_viewport: CursorViewportArc,
  message_viewport: ViewportArc,
}

impl CommandLine {
  pub fn new(shape: IRect, text_contents: TextContentsWk) -> Self {
    // Force cmdline window options.
    let options = WindowOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0_u16)
      .build()
      .unwrap();

    let root = RootContainer::new(shape);
    let root_id = root.id();
    let root_node = CommandLineNode::RootContainer(root);

    let mut base = Itree::new(root_node);

    let indicator_shape =
      IRect::new(shape.min().into(), (shape.min().x + 1, shape.max().y));
    let indicator = Indicator::new(indicator_shape, IndicatorSymbol::Empty);
    let indicator_id = indicator.id();
    let mut indicator_node = CommandLineNode::Indicator(indicator);
    // Indicator by default is invisible
    indicator_node.set_visible(false);
    base.bounded_insert(root_id, indicator_node);

    let input_shape =
      IRect::new((shape.min().x + 1, shape.min().y), shape.max().into());

    let (input_viewport, input_cursor_viewport, message_viewport) = {
      let input_actual_shape = geo_rect_as!(input_shape, u16);
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

      let message_actual_shape = geo_rect_as!(shape, u16);
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

    let input = Input::new(
      input_shape,
      text_contents.clone(),
      Arc::downgrade(&input_viewport),
    );
    let input_id = input.id();
    let mut input_node = CommandLineNode::Input(input);
    // Input by default is invisible
    input_node.set_visible(false);
    base.bounded_insert(root_id, input_node);

    let message = Message::new(
      shape,
      text_contents.clone(),
      Arc::downgrade(&message_viewport),
    );
    let message_id = message.id();
    let message_node = CommandLineNode::Message(message);
    base.bounded_insert(root_id, message_node);

    Self {
      base,
      options,
      indicator_id,
      input_id,
      message_id,
      cursor_id: None,
      input_viewport,
      input_cursor_viewport,
      message_viewport,
    }
  }
}

inode_itree_impl!(CommandLine, base);

impl Widgetable for CommandLine {
  fn draw(&self, canvas: &mut Canvas) {
    for node in self.base.iter() {
      // trace!("Draw window:{:?}", node);
      if !node.visible() {
        continue;
      }
      node.draw(canvas);
    }
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

  /// Get message viewport.
  pub fn message_viewport(&self) -> ViewportArc {
    self.message_viewport.clone()
  }

  /// Set viewport for input.
  pub fn set_input_viewport(&mut self, viewport: ViewportArc) {
    self.input_viewport = viewport.clone();
    if let Some(CommandLineNode::Input(input)) =
      self.base.node_mut(self.input_id)
    {
      input.set_viewport(Arc::downgrade(&viewport));
    }
  }

  /// Set viewport for message.
  pub fn set_message_viewport(&mut self, viewport: ViewportArc) {
    self.message_viewport = viewport.clone();
    if let Some(CommandLineNode::Message(message)) =
      self.base.node_mut(self.message_id)
    {
      message.set_viewport(Arc::downgrade(&viewport));
    }
  }

  /// Get cursor viewport for input.
  pub fn input_cursor_viewport(&self) -> CursorViewportArc {
    self.input_cursor_viewport.clone()
  }

  /// Set cursor viewport for input.
  pub fn set_input_cursor_viewport(
    &mut self,
    cursor_viewport: CursorViewportArc,
  ) {
    self.input_cursor_viewport = cursor_viewport;
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
  pub fn input(&self) -> &Input {
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
  pub fn input_mut(&mut self) -> &mut Input {
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
  pub fn message(&self) -> &Message {
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
  pub fn message_mut(&mut self) -> &mut Message {
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
  pub fn indicator(&self) -> &Indicator {
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
  pub fn indicator_mut(&mut self) -> &mut Indicator {
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
  /// Enable/insert cursor widget in commandline, i.e. when user start command-line mode, the
  /// cursor moves to the command-line widget and allow receive user ex command or search patterns.
  ///
  /// # Returns
  /// It returns the old cursor widget if there's any, otherwise it returns `None`.
  pub fn insert_cursor(&mut self, cursor: Cursor) -> Option<CommandLineNode> {
    self.cursor_id = Some(cursor.id());
    self
      .base
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
        debug_assert!(self.base.node(cursor_id).is_some());
        debug_assert!(self.base.parent_id(cursor_id).is_some());
        debug_assert_eq!(
          self.base.parent_id(cursor_id).unwrap(),
          self.input_id
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
        debug_assert!(self.base.node(self.input_id).is_some());
        debug_assert!(self.base.children_ids(self.input_id).is_empty());
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
