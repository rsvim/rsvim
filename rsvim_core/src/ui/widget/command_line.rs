//! Vim command-line widget.

#![allow(dead_code)]

use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{CursorViewport, CursorViewportArc, Viewport, ViewportArc, Viewportable};
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::{WindowLocalOptions, WindowLocalOptionsBuilder};
use crate::{inode_impl, lock};

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: InodeBase,

  options: WindowLocalOptions,

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

    let base = InodeBase::new(shape);
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
  /// Get text contents.
  pub fn contents(&self) -> TextContentsWk {
    self.contents.clone()
  }
}
// Attributes }
