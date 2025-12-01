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

pub const BLINKING: bool = false;
pub const HIDDEN: bool = false;
pub const STYLE: CursorStyle = CursorStyle::SteadyBlock;

flags_impl!(Flags, u8, BLINKING, HIDDEN);

#[derive(Debug, Clone)]
/// Cursor widget.
pub struct Cursor {
  base: InodeBase,
  // blinking=false
  // hidden=false
  flags: Flags,
  cursor_style: CursorStyle,
}

impl Cursor {
  pub fn new(
    lotree: ItreeWk,
    id: TreeNodeId,
    blinking: bool,
    hidden: bool,
    cursor_style: CursorStyle,
  ) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Self {
      base: InodeBase::new(lotree, id),
      flags,
      cursor_style,
    }
  }

  pub fn default(lotree: ItreeWk, id: TreeNodeId) -> Self {
    Self::new(lotree, id, BLINKING, HIDDEN, STYLE)
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

  pub fn cursor_style(&self) -> &CursorStyle {
    &self.cursor_style
  }

  pub fn set_cursor_style(&mut self, style: &CursorStyle) {
    self.cursor_style = *style;
  }
}

inode_impl!(Cursor);

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
      *self.cursor_style(),
    ));
  }
}
