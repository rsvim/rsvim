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
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use taffy::TaffyResult;

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: InodeBase,
  options: WindowOptions,

  indicator_id: TreeNodeId,
  input_id: TreeNodeId,
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

      // When creating "command-line" widget, "input" and "indicator" node is not attached to parent
      // yet, thus their actual_shape are all zero. So here we simply mock a shape for viewport
      // calculation.
      // Don't worry, when "command-line" switches to input, we will calculate the real shape for
      // input/indicator widget.
      let message_actual_shape = lotree.actual_shape(message_id)?;

      let text_contents = text_contents.upgrade().unwrap();
      let text_contents = lock!(text_contents);
      let input_viewport = Viewport::view(
        &options,
        text_contents.command_line_input(),
        &message_actual_shape,
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

// // Cursor {
// impl CommandLine {
//   /// Set cursor ID in commandline, e.g. user starts command-line mode, and
//   /// cursor moves into the command-line widget.
//   ///
//   /// # Returns
//   /// It returns the previous cursor ID.
//   pub fn set_cursor_id(&mut self, cursor_id: TreeNodeId) -> Option<TreeNodeId> {
//     let old = self.cursor_id;
//     self.cursor_id = Some(cursor_id);
//     old
//   }
//
//   /// Clear cursor widget from commandline, e.g. user leaves command-line mode,
//   /// cursor moves out of command-line widget.
//   ///
//   /// # Returns
//   /// It returns the previous cursor ID.
//   pub fn clear_cursor_id(&mut self) -> Option<TreeNodeId> {
//     let old = self.cursor_id;
//     self.cursor_id = None;
//     old
//   }
// }
// // Cursor }
