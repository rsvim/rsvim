//! Vim window's text content widget.

use crate::buf::{Buffer, BufferWk};
use crate::cart::{IRect, U16Pos, U16Rect, U16Size};
use crate::envar;
use crate::inode_generate_impl;
use crate::ui::canvas::internal::iframe::Iframe;
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeBase, InodeId, Inodeable};
use crate::ui::tree::Tree;
use crate::ui::util::ptr::SafeWindowRef;
use crate::ui::widget::window::{Window, WindowLocalOptions};
use crate::ui::widget::Widgetable;

use crossterm::style::{Attributes, Color};
use geo::point;
use icu::segmenter::WordSegmenter;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::time::Duration;
use tracing::{debug, error};

#[derive(Debug, Clone)]
/// The widget contains text contents for Vim window.
pub struct WindowContent {
  base: InodeBase,

  window_ref: SafeWindowRef,
}

impl WindowContent {
  /// Make window content.
  pub fn new(shape: IRect, window: &mut Window) -> Self {
    let base = InodeBase::new(shape);
    WindowContent {
      base,
      window_ref: SafeWindowRef::new(window),
    }
  }
}

inode_generate_impl!(WindowContent, base);

impl Widgetable for WindowContent {
  fn draw(&mut self, canvas: &mut Canvas) {}
}

#[cfg(test)]
mod tests {
  use super::*;

  use crate::buf::BufferArc;
  use crate::cart::U16Size;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;
}
