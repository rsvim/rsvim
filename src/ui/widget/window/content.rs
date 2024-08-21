//! VIM window's text content widget.

#![allow(unused_imports, dead_code)]

use compact_str::CompactString;
use std::collections::VecDeque;
use std::convert::From;
use std::marker::{PhantomData, PhantomPinned};
use std::ptr::NonNull;
use tracing::debug;

use crate::buffer::Buffer;
use crate::cart::{IRect, U16Rect};
use crate::inode_value_generate_impl;
use crate::ui::canvas::Canvas;
use crate::ui::tree::internal::inode::{Inode, InodeId, InodeValue};
use crate::ui::widget::{Widget, WidgetId};
use crate::uuid;

#[derive(Debug, Copy, Clone)]
struct BufferView {
  pub lstart: Option<usize>,
  pub lend: Option<usize>,
  pub cstart: Option<usize>,
  pub cend: Option<usize>,
}

impl BufferView {
  pub fn new() -> Self {
    BufferView {
      lstart: None,
      lend: None,
      cstart: None,
      cend: None,
    }
  }
}

impl Default for BufferView {
  fn default() -> Self {
    BufferView::new()
  }
}

#[derive(Debug, Clone)]
/// The content of the VIM window.
///
/// Besides buffer and window, here introduce several terms and concepts:
///
/// * Line: One line of text content in a buffer.
/// * Row/column: The width/height of a window.
/// * View: A window only shows part of a buffer when the buffer is too big to put all the text
///   contents in the window. When a buffer shows in a window, thus the window starts and ends at
///   specific lines and columns of the buffer.
///
/// There are two options related to the view:
/// [line-wrap and word-wrap](https://en.wikipedia.org/wiki/Line_wrap_and_word_wrap), so we have 4
/// kinds of views.
///
/// * Both line-wrap and word-wrap enabled.
/// * Line-wrap enabled and word-wrap disabled.
/// * Line-wrap disabled and word-wrap enabled.
/// * Both Line-wrap and word-wrap disabled.
///
/// For the first 3 kinds of view, when a window that has `X` rows height, it may contains less
/// than `X` lines of a buffer. Because very long lines or words can take extra spaces and trigger
/// line breaks. The real lines the window can contain needs a specific algorithm to calculate.
///
/// For the last kind of view, it contains exactly `X` lines of a buffer at most, but the lines
/// longer than the window's width are truncated by the window's boundary.
///
/// A view contains 4 fields:
///
/// * Start line (`lstart`).
/// * End line (`lend`).
/// * Start column (`cstart`).
/// * End column (`cend`).
///
/// We can always calculates the two fields based on the other two fields on the diagonal corner,
/// with window size, buffer's text contents, and the line-wrap/word-wrap options.
pub struct WindowContent<'a> {
  base: Inode,

  // Buffer reference
  buffer: NonNull<Buffer>,
  phantom: PhantomData<&'a mut Buffer>,

  // Buffer view
  buffer_view: BufferView,

  // Options
  line_wrap: bool,
  word_wrap: bool,
}

impl<'a> WindowContent<'a> {
  pub fn new(shape: IRect, buffer: &'a mut Buffer) -> Self {
    WindowContent {
      base: Inode::new(shape),
      buffer: NonNull::new(buffer as *mut Buffer).unwrap(),
      phantom: PhantomData,
      buffer_view: BufferView::default(),
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
    unsafe { self.buffer.as_ref() }
  }

  pub fn buffer_mut(&mut self) -> &mut Buffer {
    unsafe { self.buffer.as_mut() }
  }

  pub fn buffer_view_lstart(&self) -> Option<usize> {
    self.buffer_view.lstart
  }

  pub fn set_buffer_view_lstart(&mut self, lstart: Option<usize>) {
    self.buffer_view.lstart = lstart;
  }

  pub fn buffer_view_lend(&self) -> Option<usize> {
    self.buffer_view.lend
  }

  pub fn set_buffer_view_lend(&mut self, lend: Option<usize>) {
    self.buffer_view.lend = lend;
  }

  pub fn buffer_view_cstart(&self) -> Option<usize> {
    self.buffer_view.cstart
  }

  pub fn set_buffer_view_cstart(&mut self, cstart: Option<usize>) {
    self.buffer_view.cstart = cstart;
  }

  pub fn buffer_view_cend(&self) -> Option<usize> {
    self.buffer_view.cend
  }

  pub fn set_buffer_view_cend(&mut self, cend: Option<usize>) {
    self.buffer_view.cend = cend;
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
