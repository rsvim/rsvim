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

#[derive(Debug, Clone, Copy)]
pub struct CursorOptions {
  // blinking=false
  // hidden=false
  flags: Flags,
  style: CursorStyle,
}

impl CursorOptions {
  pub fn new(blinking: bool, hidden: bool, style: CursorStyle) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Self { flags, style }
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

  pub fn set_style(&mut self, style: &CursorStyle) {
    self.style = *style;
  }
}

impl Default for CursorOptions {
  fn default() -> Self {
    Self::new(false, false, CursorStyle::SteadyBlock)
  }
}

#[derive(Debug, Clone)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  options: CursorOptions,
}

impl Cursor {
  pub fn new(id: TreeNodeId, shape: U16Rect, options: CursorOptions) -> Self {
    Cursor {
      base: InodeBase::new(id, shape),
      options,
    }
  }

  pub fn default(loid: TreeNodeId, shape: U16Rect) -> Self {
    Self::new(loid, shape, CursorOptions::default())
  }

  pub fn options(&self) -> &CursorOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: CursorOptions) {
    self.options = options;
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
      self.options().blinking(),
      self.options().hidden(),
      *self.options().style(),
    ));
  }
}
