//! Cursor widget.

use crate::inode_impl;
use crate::prelude::*;
use crate::ui::canvas;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CursorStyle;
use crate::ui::tree::*;
use crate::ui::widget::Widgetable;
use bitflags::bitflags;
use std::fmt::Debug;

bitflags! {
  #[derive(Copy, Clone)]
  struct Flags: u8{
    const BLINKING = 1;
    const HIDDEN = 1 << 1;
  }
}

impl Debug for Flags {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Flags")
      .field("bits", &format!("{:b}", self.bits()))
      .finish()
  }
}

#[derive(Debug, Clone, Copy)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  style: CursorStyle,
  // blinking
  // hidden
  flags: Flags,
}

impl Cursor {
  pub fn new(
    shape: IRect,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    let mut flags = Flags::empty();
    if blinking {
      flags.insert(Flags::BLINKING);
    }
    if hidden {
      flags.insert(Flags::HIDDEN);
    }
    Cursor {
      base: InodeBase::new(shape),
      style,
      flags,
    }
  }

  pub fn default(shape: IRect) -> Self {
    // blinking=false
    // hidden=false
    let flags = Flags::empty();
    Cursor {
      base: InodeBase::new(shape),
      style: CursorStyle::SteadyBlock,
      flags,
    }
  }

  pub fn blinking(&self) -> bool {
    self.flags.contains(Flags::BLINKING)
  }

  pub fn set_blinking(&mut self, value: bool) {
    if value {
      self.flags.insert(Flags::BLINKING);
    } else {
      self.flags.remove(Flags::BLINKING);
    }
  }

  pub fn hidden(&self) -> bool {
    self.flags.contains(Flags::HIDDEN)
  }

  pub fn set_hidden(&mut self, value: bool) {
    if value {
      self.flags.insert(Flags::HIDDEN);
    } else {
      self.flags.remove(Flags::HIDDEN);
    }
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
