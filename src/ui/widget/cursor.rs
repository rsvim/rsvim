//! Cursor widget.

use crate::cart::{IRect, U16Pos, U16Rect};
use crate::inode_generate_impl;
use crate::ui::canvas::{self, Canvas, CursorStyle, CursorStyleFormatter};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::tree::GlobalOptions;
use crate::ui::widget::Widgetable;

use std::fmt::Debug;
use tracing::debug;

#[derive(Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(shape: IRect) -> Self {
    Cursor {
      base: InodeBase::new(shape),
      blinking: true,
      hidden: false,
      style: CursorStyle::DefaultUserShape,
    }
  }
}

impl Debug for Cursor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let style_formatter = CursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("id", &self.base.id())
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("style", &style_formatter)
      .finish()
  }
}

inode_generate_impl!(Cursor, base);

impl Widgetable for Cursor {
  fn draw(&mut self, canvas: &mut Canvas, _global_options: &GlobalOptions) {
    let actual_shape = self.actual_shape();
    let pos: U16Pos = actual_shape.min().into();
    debug!(
      "draw, actual shape:{:?}, top-left pos:{:?}",
      actual_shape, pos
    );

    canvas.frame_mut().set_cursor(canvas::Cursor::new(
      pos,
      self.blinking,
      self.hidden,
      self.style,
    ));
  }
}
