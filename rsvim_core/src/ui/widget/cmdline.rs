//! Command-line widget.

pub mod indicator;
pub mod input;
pub mod message;

#[cfg(test)]
pub mod indicator_tests;

use crate::content::TextContentsWk;
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
