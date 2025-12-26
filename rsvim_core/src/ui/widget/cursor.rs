//! Cursor widget.

use crate::flags_impl;
use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use std::fmt::Debug;

flags_impl!(Flags, u8, BLINKING, HIDDEN);

pub const CURSOR_BLINKING: bool = false;
pub const CURSOR_HIDDEN: bool = false;
pub const CURSOR_STYLE: CursorStyle = CursorStyle::SteadyBlock;

#[derive(Debug, Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  __node: InodeBase,
  // blinking=false
  // hidden=false
  flags: Flags,
  style: CursorStyle,
}

inode_impl!(Cursor);

impl Cursor {
  pub fn new(
    id: TreeNodeId,
    ctx: TreeContextRc,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Cursor {
      __node: InodeBase::new(id, ctx),
      flags,
      style,
    }
  }

  pub fn default(shape: IRect) -> Self {
    Self::new(shape, CURSOR_BLINKING, CURSOR_HIDDEN, CURSOR_STYLE)
  }

  pub fn blinking(&self) -> bool {
    self.flags.contains(Flags::BLINKING)
  }

  pub fn set_blinking(&mut self, value: bool) {
    self.flags.set(Flags::BLINKING, value);
  }

  pub fn hidden(&self) -> bool {
    self.flags.contains(Flags::HIDDEN)
  }

  pub fn set_hidden(&mut self, value: bool) {
    self.flags.set(Flags::HIDDEN, value);
  }

  pub fn style(&self) -> &CursorStyle {
    &self.style
  }

  pub fn set_style(&mut self, value: &CursorStyle) {
    self.style = *value;
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
      self.style,
    ));
  }
}
