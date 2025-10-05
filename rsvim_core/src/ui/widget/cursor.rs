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

flags_impl!(Flags, u8, BLINKING, 0b0000_0001, HIDDEN, 0b0000_0010);

#[derive(Debug, Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  // blinking=false
  // hidden=false
  flags: Flags,
  style: CursorStyle,
}

impl Cursor {
  pub fn new(
    shape: IRect,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Cursor {
      base: InodeBase::new(shape),
      flags,
      style,
    }
  }

  pub fn default(shape: IRect) -> Self {
    Cursor {
      base: InodeBase::new(shape),
      // blinking=false
      // hidden=false
      flags: Flags::empty(),
      style: CursorStyle::SteadyBlock,
    }
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

inode_impl!(Cursor, base);

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
