//! The VIM window.

use crate::buf::BufferWk;
use crate::cart::{IRect, U16Pos, U16Rect};
use crate::ui::canvas::{Canvas, Cell};
use crate::ui::tree::internal::{InodeId, Inodeable, Itree};
use crate::ui::tree::GlobalOptions;
use crate::ui::widget::window::content::WindowContent;
use crate::ui::widget::window::root::WindowRootContainer;
use crate::ui::widget::Widgetable;
use crate::{defaults, glovar};

// Re-export
pub use crate::ui::widget::window::opt::{WindowLocalOptions, WindowOptionsBuilder};

use crossterm::style::{Attributes, Color};
use geo::point;
use regex::Regex;
use ropey::RopeSlice;
use std::collections::{BTreeSet, VecDeque};
use std::convert::From;
use std::ptr::NonNull;
use std::time::Duration;
use tracing::{debug, error};

pub mod content;
pub mod opt;
pub mod root;

#[derive(Debug, Copy, Clone, Default)]
/// The view of a buffer. The range is left-inclusive right-exclusive, or top-inclusive
/// bottom-exclusive, i.e. `[start_line, end_line)` or `[start_column, end_column)`.
struct BufferView {
  /// Start line number
  pub start_line: Option<usize>,
  /// End line number.
  pub end_line: Option<usize>,
  /// Start column.
  pub start_column: Option<usize>,
  /// End column.
  pub end_column: Option<usize>,
}

impl BufferView {
  pub fn new(
    start_line: Option<usize>,
    end_line: Option<usize>,
    start_column: Option<usize>,
    end_column: Option<usize>,
  ) -> Self {
    BufferView {
      start_line,
      end_line,
      start_column,
      end_column,
    }
  }
}

#[derive(Debug, Clone)]
/// The Vim window, it manages all descendant widget nodes, i.e. all widgets in the
/// [`crate::ui::widget::window`] module.
///
/// For the window content, here introduce several terms and concepts:
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
/// * Start line.
/// * End line.
/// * Start column.
/// * End column.
///
/// We can always calculates the two fields based on the other two fields on the diagonal corner,
/// with window size, buffer's text contents, and the line-wrap/word-wrap options.
pub struct Window {
  base: Itree<WindowNode>,

  // The Window content widget ID.
  content_id: InodeId,

  // Buffer
  buffer: BufferWk,
  // Buffer view
  view: BufferView,

  // Local window options.
  // By default these options will inherit from global options of UI.
  options: WindowLocalOptions,
}

impl Window {
  pub fn new(shape: IRect, buffer: BufferWk, global_options: &GlobalOptions) -> Self {
    let options = global_options.window_local_options.clone();
    let view = BufferView::new(Some(0), None, Some(0), Some(shape.width() as usize));

    let window_root = WindowRootContainer::new(shape);
    let window_root_id = window_root.id();
    let window_root_node = WindowNode::WindowRootContainer(window_root);

    let mut base = Itree::new(window_root_node);

    let window_content = WindowContent::new(shape);
    let window_content_id = window_content.id();
    let window_content_node = WindowNode::WindowContent(window_content);

    base.bounded_insert(&window_root_id, window_content_node);
    match base.node_mut(&window_content_id).unwrap() {
      WindowNode::WindowContent(ref mut content) => content.sync_frame_size(),
      _ => unreachable!("Failed to query window content"),
    }

    Window {
      base,
      content_id: window_content_id,
      buffer,
      view,
      options,
    }
  }
}

impl Inodeable for Window {
  fn id(&self) -> InodeId {
    self.base.root_id()
  }

  fn depth(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().depth()
  }

  fn depth_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .depth_mut()
  }

  fn zindex(&self) -> &usize {
    self.base.node(&self.base.root_id()).unwrap().zindex()
  }

  fn zindex_mut(&mut self) -> &mut usize {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .zindex_mut()
  }

  fn shape(&self) -> &IRect {
    self.base.node(&self.base.root_id()).unwrap().shape()
  }

  fn shape_mut(&mut self) -> &mut IRect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .shape_mut()
  }

  fn actual_shape(&self) -> &U16Rect {
    self.base.node(&self.base.root_id()).unwrap().actual_shape()
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .actual_shape_mut()
  }

  fn enabled(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().enabled()
  }

  fn enabled_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .enabled_mut()
  }

  fn visible(&self) -> &bool {
    self.base.node(&self.base.root_id()).unwrap().visible()
  }

  fn visible_mut(&mut self) -> &mut bool {
    self
      .base
      .node_mut(&self.base.root_id())
      .unwrap()
      .visible_mut()
  }
}

#[allow(dead_code)]
fn rpslice2line(s: &RopeSlice) -> String {
  let mut builder: String = String::new();
  for chunk in s.chunks() {
    builder.push_str(chunk);
  }
  builder
}

// Draw {
impl Window {
  /// Draw buffer from `start_line`
  pub fn _draw_from_top(
    &mut self,
    content: &mut WindowContent,
    start_line: usize,
    start_column: usize,
    end_column: usize,
  ) {
    match (self.wrap(), self.line_break()) {
      (false, _) => self._draw_from_top_for_nowrap(content, start_line, start_column, end_column),
      (true, false) => {
        self._draw_from_top_for_wrap_nolinebreak(content, start_line, start_column, end_column)
      }
      (true, true) => debug!("_draw_from_top - wrap:true, line_break:true"),
    }
  }

  /// Implement the [`_draw_from_top`] with below window options:
  /// - [`warp`](WindowOptions::wrap) is `true`.
  /// - [`line_break`](WindowOptions::line_break) is `false`
  /// - [`break_at`](WindowOptions::break_at) will not be used since 'line-break' is `false`.
  pub fn _draw_from_top_for_wrap_nolinebreak(
    &mut self,
    content: &mut WindowContent,
    start_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    debug!("_draw_from_top_for_wrap_nolinebreak");
    // debug!(
    //   "actual_shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
    //   actual_shape, upos, height, width,
    // );

    // If window is zero-sized.
    if height == 0 || width == 0 {
      return;
    }

    // Get buffer arc pointer
    let buffer = self.buffer.upgrade().unwrap();

    // Lock buffer for read
    let buffer = buffer.try_read_for(glovar::MUTEX_TIMEOUT()).unwrap();

    // if let Some(line) = buffer.rope().get_line(start_line) {
    //   debug!(
    //     "buffer.get_line ({:?}):'{:?}'",
    //     start_line,
    //     rpslice2line(&line),
    //   );
    // } else {
    //   debug!("buffer.get_line ({:?}):None", start_line);
    // }

    match buffer.rope().get_lines_at(start_line) {
      Some(mut buflines) => {
        // The `start_line` is inside the buffer.
        // Render the lines from `start_line` till the end of the buffer or the window widget.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          match buflines.next() {
            Some(line) => {
              // For the row in current window widget, if has the line in buffer.
              let mut col = 0_u16;

              for chunk in line.chunks() {
                if col >= width {
                  row += 1;
                  col = 0_u16;
                  if row >= height {
                    break;
                  }
                }
                for ch in chunk.chars() {
                  if col >= width {
                    row += 1;
                    col = 0_u16;
                    if row >= height {
                      break;
                    }
                  }
                  if ch != '\n' {
                    let cell = Cell::from(ch);
                    let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
                    // debug!(
                    //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
                    //   row, col, ch, cell_upos
                    // );
                    content.frame_mut().set_cell(cell_upos, cell);
                  }
                  col += 1;
                }
              }

              // The line doesn't fill the whole row in current widget, fill left parts with empty
              // cells.
              if row < height && col < width - 1 {
                let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
                let cells_len = (width - col) as usize;
                // debug!(
                //   "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
                //   row, col, cells_upos, cells_len,
                // );
                content
                  .frame_mut()
                  .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                  .unwrap();
              }
            }
            None => {
              // If there's no more lines in the buffer, simply set the whole line to empty for
              // left parts of the window.
              let cells_upos = point!(x: upos.x(), y: row + upos.y());
              let cells_len = width as usize;
              // debug!(
              //   "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
              //   row, cells_upos, cells_len,
              // );
              content
                .frame_mut()
                .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                .unwrap();
            }
          }
          // Iterate to next row.
          row += 1;
        }
      }
      None => {
        // The `start_line` is outside of the buffer.
        // Render the whole window contents as empty cells.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          // There's no lines in the buffer, simply set the whole line to empty.
          let cells_upos = point!(x: upos.x(), y: row + upos.y());
          let cells_len = width as usize;
          // debug!(
          //   "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
          //   row, cells_upos, cells_len,
          // );
          content
            .frame_mut()
            .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
            .unwrap();
          row += 1;
        }
      }
    }
  }

  /// Implement the [`_draw_from_top`] with below options:
  /// - [`warp`](WindowOptions::wrap) is `false`.
  /// - [`line_break`](WindowOptions::line_break) and [`break_at`](WindowOptions::break_at) will
  ///   not be used since 'wrap' is `false`.
  pub fn _draw_from_top_for_nowrap(
    &mut self,
    content: &mut WindowContent,
    start_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    let actual_shape = self.actual_shape();
    let upos: U16Pos = actual_shape.min().into();
    let height = actual_shape.height();
    let width = actual_shape.width();

    debug!("_draw_from_top_for_nowrap");
    // debug!(
    //   "actual shape:{:?}, upos:{:?}, height/width:{:?}/{:?}",
    //   actual_shape, upos, height, width,
    // );

    // If window is zero-sized.
    if height == 0 || width == 0 {
      return;
    }

    // Get buffer arc pointer
    let buffer = self.buffer.upgrade().unwrap();

    // Lock buffer for read
    let buffer = buffer.try_read_for(glovar::MUTEX_TIMEOUT()).unwrap();

    // if let Some(line) = buffer.rope().get_line(start_line) {
    //   debug!(
    //     "buffer.get_line ({:?}):'{:?}'",
    //     start_line,
    //     rslice2line(&line),
    //   );
    // } else {
    //   debug!("buffer.get_line ({:?}):None", start_line);
    // }

    match buffer.rope().get_lines_at(start_line) {
      Some(mut buflines) => {
        // The `start_line` is inside the buffer.
        // Render the lines from `start_line` till the end of the buffer or the window widget.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          match buflines.next() {
            Some(line) => {
              // For the row in current window widget, if has the line in buffer.
              let mut col = 0_u16;

              for chunk in line.chunks() {
                if col >= width {
                  break;
                }
                for ch in chunk.chars() {
                  if col >= width {
                    break;
                  }
                  if ch != '\n' {
                    let cell = Cell::from(ch);
                    let cell_upos = point!(x: col + upos.x(), y: row + upos.y());
                    // debug!(
                    //   "1-row:{:?}, col:{:?}, ch:{:?}, cell upos:{:?}",
                    //   row, col, ch, cell_upos
                    // );
                    content.frame_mut().set_cell(cell_upos, cell);
                  }
                  col += 1;
                }
              }

              // The line doesn't fill the whole row in current widget, fill left parts with empty
              // cells.
              if row < height && col < width - 1 {
                let cells_upos = point!(x: col + upos.x(), y: row + upos.y());
                let cells_len = (width - col) as usize;
                // debug!(
                //   "2-row:{:?}, col:{:?}, cells upos:{:?}, cells len:{:?}",
                //   row, col, cells_upos, cells_len,
                // );
                content
                  .frame_mut()
                  .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                  .unwrap();
              }
            }
            None => {
              // If there's no more lines in the buffer, simply set the whole line to empty for
              // left parts of the window.
              let cells_upos = point!(x: upos.x(), y: row + upos.y());
              let cells_len = width as usize;
              // debug!(
              //   "3-row:{:?}, cells upos:{:?}, cells len:{:?}",
              //   row, cells_upos, cells_len,
              // );
              content
                .frame_mut()
                .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
                .unwrap();
            }
          }
          // Iterate to next row.
          row += 1;
        }
      }
      None => {
        // The `start_line` is outside of the buffer.
        // Render the whole window contents as empty cells.

        // The first `row` (0) in the window maps to the `start_line` in the buffer.
        let mut row = 0;

        while row < height {
          // There's no lines in the buffer, simply set the whole line to empty.
          let cells_upos = point!(x: upos.x(), y: row + upos.y());
          let cells_len = width as usize;
          // debug!(
          //   "4-row:{:?}, cells upos:{:?}, cells len:{:?}",
          //   row, cells_upos, cells_len,
          // );
          content
            .frame_mut()
            .try_set_cells_at(cells_upos, vec![Cell::empty(); cells_len])
            .unwrap();
          row += 1;
        }
      }
    }
  }

  /// Draw buffer from `end_line` in reverse order.
  pub fn _draw_from_bottom(
    &mut self,
    _content: &mut WindowContent,
    _end_line: usize,
    _start_column: usize,
    _end_column: usize,
  ) {
    unimplemented!()
  }
}
// Draw }

impl Widgetable for Window {
  fn draw(&mut self, canvas: &mut Canvas) {
    // Preprocessing {
    unsafe {
      let mut raw_self = NonNull::new(self as *mut Window).unwrap();
      match raw_self
        .as_mut()
        .base
        .node_mut(&raw_self.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => match self.view {
          BufferView {
            start_line: Some(start_line),
            end_line: _,
            start_column: Some(start_column),
            end_column: Some(end_column),
          } => raw_self
            .as_mut()
            ._draw_from_top(content, start_line, start_column, end_column),
          BufferView {
            start_line: _,
            end_line: Some(end_line),
            start_column: Some(start_column),
            end_column: Some(end_column),
          } => raw_self
            .as_mut()
            ._draw_from_bottom(content, end_line, start_column, end_column),
          _ => {
            error!("Invalid buffer view: {:?}", self.view);
            unreachable!("Invalid buffer view")
          }
        },
        _ => unreachable!("Failed to query window content node"),
      }
    }
    // Preprocessing }

    for node in self.base.iter_mut() {
      debug!("draw node:{:?}", node);
      node.draw(canvas);
    }
  }
}

// Options {
impl Window {
  pub fn options(&self) -> &WindowLocalOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &WindowLocalOptions) {
    self.options = options.clone();
  }

  pub fn wrap(&self) -> bool {
    self.options.wrap()
  }

  pub fn set_wrap(&mut self, value: bool) {
    self.options.set_wrap(value);
  }

  pub fn line_break(&self) -> bool {
    self.options.line_break()
  }

  pub fn set_line_break(&mut self, value: bool) {
    self.options.set_line_break(value);
  }

  // pub fn break_at(&self) -> &String {}
  //
  // pub fn break_at_regex(&self) -> &Regex {}
}
// Options }

// Buffer/View {
impl Window {
  /// Get buffer reference.
  pub fn buffer(&self) -> BufferWk {
    self.buffer.clone()
  }

  /// Set buffer reference.
  pub fn set_buffer(&mut self, buffer: BufferWk) {
    self.buffer = buffer;
  }

  /// Get start line, index start from 0.
  pub fn start_line(&self) -> Option<usize> {
    self.view.start_line
  }

  /// Set start line.
  ///
  /// This operation will unset the end line. Because with different line-wrap/word-wrap options,
  /// the window may contains less lines than its height. We cannot know the end line unless
  /// iterating over the buffer from start line.
  pub fn set_start_line(&mut self, line: usize) {
    self.view.start_line = Some(line);
    self.view.end_line = None;
  }

  /// Get end line, index start from 0.
  pub fn end_line(&self) -> Option<usize> {
    self.view.end_line
  }

  /// Set end line.
  ///
  /// This operation will unset the start line. Because with different line-wrap/word-wrap options,
  /// the window may contains less lines than the height. We cannot know the start line unless
  /// reversely iterating over the buffer from end line.
  pub fn set_end_line(&mut self, lend: usize) {
    self.view.end_line = Some(lend);
    self.view.start_line = None;
  }

  /// Get start column, index start from 0.
  pub fn start_column(&self) -> Option<usize> {
    self.view.start_column
  }

  /// Set start column.
  ///
  /// This operation also calculates the end column based on widget's width, and set it as well.
  pub fn set_start_column(&mut self, cstart: usize) {
    self.view.start_column = Some(cstart);
    let actual_shape = self.base.node(&self.content_id).unwrap().actual_shape();
    self.view.end_column = Some(cstart + actual_shape.width() as usize);
  }

  /// Get end column, index start from 0.
  pub fn end_column(&self) -> Option<usize> {
    self.view.end_column
  }

  /// Set end column.
  ///
  /// This operation also calculates the start column based on widget's width, and set it as well.
  pub fn set_end_column(&mut self, cend: usize) {
    self.view.end_column = Some(cend);
    let actual_shape = self.base.node(&self.content_id).unwrap().actual_shape();
    self.view.start_column = Some(cend - actual_shape.width() as usize);
  }
}
// Buffer/View }

#[derive(Debug, Clone)]
/// The value holder for each window widget.
pub enum WindowNode {
  WindowRootContainer(WindowRootContainer),
  WindowContent(WindowContent),
}

macro_rules! window_node_generate_dispatch {
  ($self_name:ident,$method_name:ident) => {
    match $self_name {
      WindowNode::WindowRootContainer(n) => n.$method_name(),
      WindowNode::WindowContent(n) => n.$method_name(),
    }
  };
}

impl Inodeable for WindowNode {
  fn id(&self) -> InodeId {
    window_node_generate_dispatch!(self, id)
  }

  fn depth(&self) -> &usize {
    window_node_generate_dispatch!(self, depth)
  }

  fn depth_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, depth_mut)
  }

  fn zindex(&self) -> &usize {
    window_node_generate_dispatch!(self, zindex)
  }

  fn zindex_mut(&mut self) -> &mut usize {
    window_node_generate_dispatch!(self, zindex_mut)
  }

  fn shape(&self) -> &IRect {
    window_node_generate_dispatch!(self, shape)
  }

  fn shape_mut(&mut self) -> &mut IRect {
    window_node_generate_dispatch!(self, shape_mut)
  }

  fn actual_shape(&self) -> &U16Rect {
    window_node_generate_dispatch!(self, actual_shape)
  }

  fn actual_shape_mut(&mut self) -> &mut U16Rect {
    window_node_generate_dispatch!(self, actual_shape_mut)
  }

  fn enabled(&self) -> &bool {
    window_node_generate_dispatch!(self, enabled)
  }

  fn enabled_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, enabled_mut)
  }

  fn visible(&self) -> &bool {
    window_node_generate_dispatch!(self, visible)
  }

  fn visible_mut(&mut self) -> &mut bool {
    window_node_generate_dispatch!(self, visible_mut)
  }
}

impl Widgetable for WindowNode {
  /// Draw widget on the canvas.
  fn draw(&mut self, canvas: &mut Canvas) {
    match self {
      WindowNode::WindowRootContainer(w) => w.draw(canvas),
      WindowNode::WindowContent(w) => w.draw(canvas),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use compact_str::ToCompactString;
  use ropey::{Rope, RopeBuilder};
  use std::fs::File;
  use std::io::{BufReader, BufWriter};
  use std::sync::Arc;
  use std::sync::Once;
  use tracing::info;

  use crate::buf::{Buffer, BufferArc};
  use crate::cart::U16Size;
  #[allow(dead_code)]
  use crate::test::log::init as test_log_init;

  #[allow(dead_code)]
  static INIT: Once = Once::new();

  fn make_buffer_from_file(filename: String) -> BufferArc {
    let rop: Rope = Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap();
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_buffer_from_lines(lines: Vec<&str>) -> BufferArc {
    let mut rop: RopeBuilder = RopeBuilder::new();
    for line in lines.iter() {
      rop.append(line);
    }
    let buf: Buffer = Buffer::from(rop);
    Buffer::to_arc(buf)
  }

  fn make_empty_buffer() -> BufferArc {
    let buf: Buffer = RopeBuilder::new().into();
    Buffer::to_arc(buf)
  }

  #[test]
  fn _draw_from_top_for_nowrap1() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);

    let expect = vec![
      "Hello, RSV",
      "This is a ",
      "But still ",
      "  1. When ",
      "  2. When ",
      "     * The",
      "     * The",
      "          ",
      "          ",
      "          ",
    ];

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    global_options.window_local_options = window_options;
    let window_shape = IRect::new((0, 0), (10, 10));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);
    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          raw_window
            .as_mut()
            ._draw_from_top_for_nowrap(content, 0, 0, 10);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          info!("expect:{:?}", expect);
          assert_eq!(actual.len(), 10);
          assert!(expect.len() <= 10);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 10);
            if i < expect.len() {
              let e = expect[i];
              info!("{:?} a:{:?}, e:{:?}", i, a, e);
              assert!(a.len() == e.len() || e.is_empty());
              if a.len() == e.len() {
                assert_eq!(a, e);
              }
            } else {
              info!("{:?} a:{:?}, e:empty", i, a);
              assert_eq!(a, [" "; 10].join(""));
            }
          }
        }
        _ => unreachable!(),
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap2() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite simple and ",
      "But still it contains sever",
      "  1. When the line is small",
      "  2. When the line is too l",
      "     * The extra parts are ",
      "     * The extra parts are ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
      "                           ",
    ];

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    global_options.window_local_options = window_options;

    let window_shape = IRect::new((0, 0), (27, 15));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);

    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          raw_window
            .as_mut()
            ._draw_from_top_for_nowrap(content, 1, 0, 0);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          info!("expect:{:?}", expect);
          assert_eq!(actual.len(), 15);
          assert!(expect.len() <= 15);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 27);
            if i < expect.len() {
              let e = expect[i];
              info!("{:?} a:{:?}, e:{:?}", i, a, e);
              assert!(a.len() == e.len() || e.is_empty());
              if a.len() == e.len() {
                assert_eq!(a, e);
              }
            } else {
              info!("{:?} a:{:?}, e:empty", i, a);
              assert_eq!(a, [" "; 27].join(""));
            }
          }
        }
        _ => unreachable!(),
      }
    }
  }

  #[test]
  fn _draw_from_top_for_nowrap3() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(false).build();
    global_options.window_local_options = window_options;

    let window_shape = IRect::new((0, 0), (20, 18));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);

    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          raw_window
            .as_mut()
            ._draw_from_top_for_nowrap(content, 0, 0, 0);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          assert_eq!(actual.len(), 18);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 20);
            info!("{:?} a:{:?}", i, a);
            assert!(a
              .chars()
              .filter(|c| *c != ' ')
              .collect::<Vec<_>>()
              .is_empty());
          }
        }
        _ => unreachable!(),
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak1() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "Hello, RSV",
      "IM!       ",
      "This is a ",
      "quite simp",
      "le and sma",
      "ll test li",
      "nes.      ",
      "But still ",
      "it contain",
      "s several ",
    ];

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    global_options.window_local_options = window_options;

    let window_shape = IRect::new((0, 0), (10, 10));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);

    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          raw_window
            .as_mut()
            ._draw_from_top_for_wrap_nolinebreak(content, 0, 0, 10);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          info!("expect:{:?}", expect);
          assert_eq!(actual.len(), 10);
          assert!(expect.len() <= 10);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 10);
            if i < expect.len() {
              let e = expect[i];
              info!("{:?} a:{:?}, e:{:?}", i, a, e);
              assert!(a.len() == e.len() || e.is_empty());
              if a.len() == e.len() {
                assert_eq!(a, e);
              }
            } else {
              info!("{:?} a:{:?}, e:empty", i, a);
              assert_eq!(a, [" "; 10].join(""));
            }
          }
        }
        _ => unreachable!(),
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak2() {
    // INIT.call_once(test_log_init);

    let buffer = make_buffer_from_lines(vec![
      "Hello, RSVIM!\n",
      "This is a quite simple and small test lines.\n",
      "But still it contains several things we want to test:\n",
      "  1. When the line is small enough to completely put inside a row of the window content widget, then the line-wrap and word-wrap doesn't affect the rendering.\n",
      "  2. When the line is too long to be completely put in a row of the window content widget, there're multiple cases:\n",
      "     * The extra parts are been truncated if both line-wrap and word-wrap options are not set.\n",
      "     * The extra parts are split into the next row, if either line-wrap or word-wrap options are been set. If the extra parts are still too long to put in the next row, repeat this operation again and again. This operation also eats more rows in the window, thus it may contains less lines in the buffer.\n",
    ]);
    let expect = vec![
      "This is a quite simple and ",
      "small test lines.          ",
      "But still it contains sever",
      "al things we want to test: ",
      "  1. When the line is small",
      " enough to completely put i",
      "nside a row of the window c",
      "ontent widget, then the lin",
      "e-wrap and word-wrap doesn'",
      "t affect the rendering.    ",
      "  2. When the line is too l",
      "ong to be completely put in",
      " a row of the window conten",
      "t widget, there're multiple",
      " cases:                    ",
    ];

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    global_options.window_local_options = window_options;

    let window_shape = IRect::new((0, 0), (27, 15));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);
    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          window._draw_from_top_for_wrap_nolinebreak(content, 1, 0, 0);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          info!("expect:{:?}", expect);
          assert_eq!(actual.len(), 15);
          assert!(expect.len() <= 15);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 27);
            if i < expect.len() {
              let e = expect[i];
              info!("{:?} a:{:?}, e:{:?}", i, a, e);
              assert!(a.len() == e.len() || e.is_empty());
              if a.len() == e.len() {
                assert_eq!(a, e);
              }
            } else {
              info!("{:?} a:{:?}, e:empty", i, a);
              assert_eq!(a, [" "; 27].join(""));
            }
          }
        }
        _ => unreachable!(),
      }
    }
  }

  #[test]
  fn _draw_from_top_for_wrap_nolinebreak3() {
    // INIT.call_once(test_log_init);

    let buffer = make_empty_buffer();

    let mut global_options = GlobalOptions::default();
    let window_options = WindowLocalOptions::builder().wrap(true).build();
    global_options.window_local_options = window_options;

    let window_shape = IRect::new((0, 0), (20, 18));
    let mut window = Window::new(window_shape, Arc::downgrade(&buffer), &global_options);

    unsafe {
      let mut raw_window = NonNull::new(&mut window as *mut Window).unwrap();
      match raw_window
        .as_mut()
        .base
        .node_mut(&raw_window.as_ref().content_id)
        .unwrap()
      {
        WindowNode::WindowContent(ref mut content) => {
          window._draw_from_top_for_wrap_nolinebreak(content, 0, 0, 0);
          let actual = content
            .frame()
            .raw_symbols_with_placeholder(" ".to_compact_string())
            .iter()
            .map(|cs| cs.join(""))
            .collect::<Vec<_>>();
          info!("actual:{:?}", actual);
          assert_eq!(actual.len(), 18);
          for (i, a) in actual.into_iter().enumerate() {
            assert!(a.len() == 20);
            info!("{:?} a:{:?}", i, a);
            assert!(a
              .chars()
              .filter(|c| *c != ' ')
              .collect::<Vec<_>>()
              .is_empty());
          }
        }
        _ => unreachable!(),
      }
    }
  }
}
