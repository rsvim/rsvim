//! Vim cmdline.

#![allow(dead_code)]

use crate::content::TemporaryContentsWk;
use crate::prelude::*;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowLocalOptions;

#[derive(Debug, Clone)]
/// The Vim cmdline.
pub struct Cmdline {
  base: InodeBase,

  // Temporary contents for cmdline content.
  contents: TemporaryContentsWk,

  options: WindowLocalOptions,
}

impl Cmdline {
  pub fn new(opts: &WindowLocalOptions, shape: IRect, contents: TemporaryContentsWk) -> Self {
    let options = *opts;
    let base = InodeBase::new(shape);
    Self {
      base,
      contents,
      options,
    }
  }
}

impl Widgetable for Cmdline {
  fn draw(&self, _canvas: &mut crate::ui::canvas::Canvas) {}
}
