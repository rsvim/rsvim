//! Vim command-line widget.

#![allow(dead_code)]

use crate::content::TemporaryContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::{
  CursorViewport, CursorViewportArc, Viewport, ViewportArc, ViewportOptions,
};
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::{WindowLocalOptions, WindowLocalOptionsBuilder};
use crate::{inode_impl, lock};

#[derive(Debug, Clone)]
/// The Vim command-line.
pub struct CommandLine {
  base: InodeBase,

  options: WindowLocalOptions,

  contents: TemporaryContentsWk,

  viewport: ViewportArc,

  cursor_viewport: CursorViewportArc,
}

impl CommandLine {
  pub fn new(shape: IRect, contents: TemporaryContentsWk) -> Self {
    // Force cmdline window options.
    let options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0_u16)
      .build()
      .unwrap();
    let viewport_options = ViewportOptions::from(&options);

    let base = InodeBase::new(shape);
    let cmdline_actual_shape = base.actual_shape();

    let (viewport, cursor_viewport) = {
      let contents = contents.upgrade().unwrap();
      let contents = lock!(contents);
      let viewport = Viewport::view(
        &viewport_options,
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
