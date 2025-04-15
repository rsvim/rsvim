//! Cursor widget.

use std::fmt::Debug;
use tracing::trace;

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas::{self, Canvas, CursorStyle};
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;

#[derive(Debug, Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(shape: IRect, blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    Cursor {
      base: InodeBase::new(shape),
      blinking,
      hidden,
      style,
    }
  }

  pub fn default(shape: IRect) -> Self {
    Cursor {
      base: InodeBase::new(shape),
      blinking: false,
      hidden: false,
      style: CursorStyle::SteadyBlock,
    }
  }
}

inode_impl!(Cursor, base);

impl Widgetable for Cursor {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let pos: U16Pos = actual_shape.min().into();
    trace!(
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
