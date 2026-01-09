//! Cursor widget.

use crate::inodify_impl;
use crate::prelude::*;
use crate::ui::canvas;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use std::fmt::Debug;

// Default values.
pub const BLINKING: bool = false;
pub const HIDDEN: bool = false;
pub const CURSOR_STYLE: CursorStyle = CursorStyle::SteadyBlock;

#[derive(Debug, Clone)]
/// Cursor widget.
pub struct Cursor {
  __node: InodeBase,
  blinking: bool,
  hidden: bool,
  cursor_style: CursorStyle,
}

inodify_impl!(Cursor);

impl Cursor {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextWk,
    blinking: bool,
    hidden: bool,
    cursor_style: CursorStyle,
  ) -> Self {
    Self {
      __node: InodeBase::new(id, ctx),
      blinking,
      hidden,
      cursor_style,
    }
  }

  pub fn default(id: TreeNodeId, ctx: TreeContextWk) -> Self {
    Self::new(id, ctx, BLINKING, HIDDEN, CURSOR_STYLE)
  }

  pub fn blinking(&self) -> bool {
    self.blinking
  }

  pub fn set_blinking(&mut self, value: bool) {
    self.blinking = value;
  }

  pub fn hidden(&self) -> bool {
    self.hidden
  }

  pub fn set_hidden(&mut self, value: bool) {
    self.hidden = value;
  }

  pub fn cursor_style(&self) -> &CursorStyle {
    &self.cursor_style
  }

  pub fn set_cursor_style(&mut self, value: CursorStyle) {
    self.cursor_style = value;
  }
}

impl Widgetable for Cursor {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let pos: U16Pos = actual_shape.min().into();
    // trace!(
    //   "draw, actual shape:{:?}, top-left pos:{:?}",
    //   actual_shape, pos
    // );

    canvas.frame_mut().set_cursor(canvas::Cursor::new(
      pos,
      self.blinking(),
      self.hidden(),
      self.cursor_style,
    ));
  }
}
