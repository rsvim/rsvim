//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect, U16Size};
use crate::glovar;
use crate::inode_generate_impl;
use crate::ui::canvas::internal::iframe::Iframe;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::widget::window::WindowLocalOptions;
use crate::ui::widget::Widgetable;

use crossterm::style::{Attributes, Color};
use geo::point;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Debug, Clone)]
/// The text contents of Vim window.
pub struct WindowContent {
  base: InodeBase,
  frame: Iframe,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect) -> Self {
    WindowContent {
      base: InodeBase::new(shape),
      // NOTE: When create this struct, it doesn't know itself actual shape. The actual shape
      // will be update when inserted into its parent node.
      frame: Iframe::new(U16Size::new(0, 0)),
    }
  }

  /// Call this method only after it's been inserted to parent node, or shape been changed.
  pub fn sync_frame_size(&mut self) {
    self
      .frame
      .set_size(U16Size::from(*self.base.actual_shape()));
  }

  pub fn frame(&self) -> &Iframe {
    &self.frame
  }

  pub fn frame_mut(&mut self) -> &mut Iframe {
    &mut self.frame
  }
}

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {
    for row in 0..self.actual_shape().height() {
      for col in 0..self.actual_shape().width() {
        let pos = U16Pos::new(col, row);
        canvas
          .frame_mut()
          .set_cell(pos, self.frame.get_cell(pos).clone());
      }
    }
  }
}

#[cfg(test)]
mod tests {}
