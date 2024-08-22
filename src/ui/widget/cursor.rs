//! Cursor widget.

use std::fmt::Debug;
use tracing::debug;

use crate::cart::{IRect, U16Pos, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::canvas::{frame, Canvas, CursorStyle, CursorStyleFormatter};
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::widget::{Widget, WidgetId};

#[derive(Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  base: Inode,
  blinking: bool,
  hidden: bool,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(shape: IRect) -> Self {
    Cursor {
      base: Inode::new(shape),
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

inode_value_generate_impl!(Cursor, base);

impl Widget for Cursor {
  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {
    let pos: U16Pos = actual_shape.min().into();
    debug!(
      "draw, actual shape:{:?}, top-left pos:{:?}",
      actual_shape, pos
    );

    canvas.frame_mut().set_cursor(frame::Cursor::new(
      pos,
      self.blinking,
      self.hidden,
      self.style,
    ));
  }
}
