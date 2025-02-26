//! Cursor widget.

use std::fmt::Debug;
use tracing::trace;

use crate::cart::{IRect, U16Pos, U16Rect};
use crate::inode_generate_impl;
use crate::ui::canvas::{CCursor, CCursorStyle, CCursorStyleFormatter, Canvas};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::widget::Widgetable;

#[derive(Clone, Copy)]
/// Cursor widget.
pub struct WCursor {
  base: InodeBase,
  blinking: bool,
  hidden: bool,
  style: CCursorStyle,
}

impl WCursor {
  pub fn new(shape: IRect) -> Self {
    WCursor {
      base: InodeBase::new(shape),
      blinking: true,
      hidden: false,
      style: CCursorStyle::DefaultUserShape,
    }
  }
}

impl Debug for WCursor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let style_formatter = CCursorStyleFormatter::from(self.style);
    f.debug_struct("Cursor")
      .field("id", &self.base.id())
      .field("blinking", &self.blinking)
      .field("hidden", &self.hidden)
      .field("style", &style_formatter)
      .finish()
  }
}

inode_generate_impl!(WCursor, base);

impl Widgetable for WCursor {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let pos: U16Pos = actual_shape.min().into();
    trace!(
      "draw, actual shape:{:?}, top-left pos:{:?}",
      actual_shape,
      pos
    );

    canvas
      .frame_mut()
      .set_cursor(CCursor::new(pos, self.blinking, self.hidden, self.style));
  }
}
