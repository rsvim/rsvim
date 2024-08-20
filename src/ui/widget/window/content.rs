//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::CompactString;
use std::collections::VecDeque;
use std::convert::From;
use tracing::debug;

use crate::buffer::{Buffer, BufferView};
use crate::cart::{IRect, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Clone)]
/// The content of the VIM window.
///
/// There are some methods related the the `line`. The `line` here is a logical concept related to
/// normal lines in the files user editing. Not the rows that render to the terminal.
///
/// Note: For a window with **N** rows height, it can contains at most **N** lines (including the
/// empty lines). If there is some very long line, and the `line_wrap` option is set, then 1
/// logical line can actually take more than 1 rows height.
pub struct WindowContent<'a> {
  base: Inode,

  // Buffer view
  buffer: &'a Buffer,
  buffer_view: BufferView,

  // Options
  line_wrap: bool,
  word_wrap: bool,
}

impl<'a> WindowContent<'a> {
  pub fn new(shape: IRect, buffer: &'a Buffer, buffer_view: BufferView) -> Self {
    WindowContent {
      base: Inode::new(shape),
      buffer,
      buffer_view,
      line_wrap: false,
      word_wrap: false,
    }
  }

  pub fn line_wrap(&self) -> bool {
    self.line_wrap
  }

  pub fn set_line_wrap(&mut self, line_wrap: bool) {
    self.line_wrap = line_wrap;
  }

  pub fn word_wrap(&self) -> bool {
    self.word_wrap
  }

  pub fn set_word_wrap(&mut self, word_wrap: bool) {
    self.word_wrap = word_wrap;
  }

  pub fn buffer(&self) -> &Buffer {
    &self.buffer
  }

  pub fn set_buffer_mut(&mut self, buffer: &'a Buffer) {
    self.buffer = buffer;
  }

  pub fn buffer_view(&self) -> &BufferView {
    &self.buffer_view
  }

  pub fn set_buffer_view(&mut self, buffer_view: BufferView) {
    self.buffer_view = buffer_view;
  }
}

impl<'a> InodeValue for WindowContent<'a> {
  fn id(&self) -> InodeId {
    self.base.id()
  }

  fn depth(&self) -> &usize {
    self.base.depth()
  }

  fn depth_mut(&mut self) -> &mut usize {
    self.base.depth_mut()
  }

  fn zindex(&self) -> &usize {
    self.base.zindex()
  }

  fn zindex_mut(&mut self) -> &mut usize {
    self.base.zindex_mut()
  }

  fn shape(&self) -> &IRect {
    self.base.shape()
  }

  fn shape_mut(&mut self) -> &mut IRect {
    self.base.shape_mut()
  }

  fn actual_shape(&self) -> &U16Rect {
    self.base.actual_shape()
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    self.base.actual_shape_mut()
  }

  fn enabled(&self) -> &bool {
    self.base.enabled()
  }

  fn enabled_mut(&mut self) -> &mut bool {
    self.base.enabled_mut()
  }

  fn visible(&self) -> &bool {
    self.base.visible()
  }

  fn visible_mut(&mut self) -> &mut bool {
    self.base.visible_mut()
  }
}

impl<'a> Widget for WindowContent<'a> {
  fn id(&self) -> WidgetId {
    self.base.id()
  }

  fn draw(&mut self, actual_shape: U16Rect, canvas: &mut Canvas) {}
}
