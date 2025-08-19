//! Commandline's input content widget.

use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use crate::{inode_impl, lock};

#[derive(Debug, Clone)]
/// Commandline input content.
pub struct Input {
  base: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

impl Input {
  pub fn new(
    shape: IRect,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    let base = InodeBase::new(shape);
    Input {
      base,
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }
}

inode_impl!(Input, base);

impl Widgetable for Input {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.text_contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(contents.command_line_content(), actual_shape, canvas);
  }
}
