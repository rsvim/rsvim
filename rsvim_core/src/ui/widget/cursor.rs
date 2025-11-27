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

#[derive(Debug, Clone)]
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
    relationship: IrelationshipRc,
    id: TreeNodeId,
    blinking: bool,
    hidden: bool,
    style: CursorStyle,
  ) -> Self {
    let mut flags = Flags::empty();
    flags.set(Flags::BLINKING, blinking);
    flags.set(Flags::HIDDEN, hidden);
    Self {
      base: InodeBase::new(relationship, id),
      flags,
      style,
    }
  }

  pub fn default(relationship: IrelationshipRc, id: TreeNodeId) -> Self {
    Self::new(relationship, id, false, false, CursorStyle::SteadyBlock)
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

inode_impl!(Cursor);

impl Widgetable for Cursor {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.base.borrow().actual_shape(self.id).unwrap();
    let pos: U16Pos = actual_shape.min().into();
    // trace!(
    //   "draw, actual shape:{:?}, top-left pos:{:?}",
    //   actual_shape, pos
    // );

    canvas.frame_mut().set_cursor(canvas::Cursor::new(
      pos,
      self.blinking(),
      self.hidden(),
      *self.style(),
    ));
  }
}
