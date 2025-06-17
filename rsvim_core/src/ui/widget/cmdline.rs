//! Vim cmdline.

#![allow(dead_code)]

use crate::content::TemporaryContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::{self, WindowLocalOptions};
use crate::{inode_impl, lock};

#[derive(Debug, Clone)]
/// The Vim cmdline.
pub struct Cmdline {
  base: InodeBase,

  // Cmdline content temporary content.
  contents: TemporaryContentsWk,

  // Cmdline content viewport.
  viewport: ViewportWk,

  options: WindowLocalOptions,
}

impl Cmdline {
  pub fn new(
    opts: &WindowLocalOptions,
    shape: IRect,
    contents: TemporaryContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    let mut options = *opts;
    options.set_wrap(false);
    options.set_line_break(false);
    options.set_scroll_off(0_u16);
    let base = InodeBase::new(shape);
    Self {
      base,
      contents,
      options,
      viewport,
    }
  }
}

inode_impl!(Cmdline, base);

impl Widgetable for Cmdline {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(contents.cmdline_content(), actual_shape, canvas);
  }
}
