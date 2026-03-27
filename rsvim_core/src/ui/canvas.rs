//! Canvas.

pub mod frame;
pub mod internal;

#[cfg(test)]
mod frame_tests;

use crate::prelude::*;
use crossterm;
use crossterm::style::Attribute;
use crossterm::style::Stylize;
pub use frame::cell::*;
pub use frame::cursor::*;
pub use frame::*;
use itertools::Itertools;
use std::fmt::Debug;
use std::slice::Iter;

#[derive(Debug, Clone)]
/// Logical canvas.
///
/// It manages both the current frame and the last frame as a screenshot, and internally uses a
/// diff-algorithm to compare the TUI changes, thus only flushing the changed parts to reduce IO
/// operations.
///
/// NOTE: APIs named without `prev_` are current frame, with `prev_` are for previous frame.
pub struct Canvas {
  frame: Frame,
  prev_frame: Frame,
}

arc_mutex_ptr!(Canvas);

impl Canvas {
  /// Make new canvas with terminal actual size.
  pub fn new(size: U16Size) -> Self {
    Canvas {
      prev_frame: Frame::new(size, Cursor::default()),
      frame: Frame::new(size, Cursor::default()),
    }
  }

  // Current frame {

  /// Get current frame.
  pub fn frame(&self) -> &Frame {
    &self.frame
  }

  /// Get mutable current frame.
  pub fn frame_mut(&mut self) -> &mut Frame {
    &mut self.frame
  }

  pub fn size(&self) -> U16Size {
    self.frame.size()
  }

  /// Get current frame cells.
  pub fn cells(&self) -> &Vec<Cell> {
    self.frame.get_cells()
  }

  /// Get current frame cursor.
  pub fn cursor(&self) -> &Cursor {
    self.frame.cursor()
  }

  // Current frame }

  // Previous frame {

  /// Get previous frame.
  pub fn prev_frame(&self) -> &Frame {
    &self.prev_frame
  }

  pub fn prev_size(&self) -> U16Size {
    self.prev_frame.size()
  }

  /// Get previous frame cells.
  pub fn prev_cells(&self) -> &Vec<Cell> {
    self.prev_frame.get_cells()
  }

  /// Get previous frame cells at specified range.
  pub fn prev_cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    self.prev_frame.get_cells_at(pos, n)
  }

  /// Get previous frame cursor.
  pub fn prev_cursor(&self) -> &Cursor {
    self.prev_frame.cursor()
  }
}

// Shade {
impl Canvas {
  /// Get the shader commands that should print to the terminal device, it internally uses a
  /// diff-algorithm to reduce the outputs.
  pub fn shade(&mut self) -> Shader {
    let mut shaders = Vec::with_capacity(
      (self.size().height() as usize) * (self.size().width() as usize),
    );

    // Hide cursor to avoid terminal cursor twinkling/jumping while rendering.
    //
    // NOTE: On Windows Terminal, flushing shaders without hiding cursor makes
    // the cursor twinkling/jumping while refreshing the TUI screen.
    // So here let's hide cursor before flushing shaders, and restore the
    // cursor after flushing is done.
    if !self.cursor().hidden() {
      shaders.push(ShaderCommand::CursorHide(crossterm::cursor::Hide));
    }

    // For cells, it needs extra save and restore cursor position
    self._shade_cells(&mut shaders);
    let saved_cursor_pos = self.cursor().pos();

    // Revert hide cursor.
    if !self.cursor().hidden() {
      shaders.push(ShaderCommand::CursorShow(crossterm::cursor::Show));
    }

    shaders.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
      saved_cursor_pos.x(),
      saved_cursor_pos.y(),
    )));

    // For cursor
    self._shade_cursor(&mut shaders);

    // Finish shade.
    self._shade_done();

    Shader::new(shaders)
  }

  /// Shade done.
  pub fn _shade_done(&mut self) {
    // Save current frame.
    self.prev_frame.clone_from(&self.frame);
    // Reset all dirty marks.
    self.frame.reset_dirty_marks();
  }

  /// Shade cursor and append results into shader vector.
  pub fn _shade_cursor(&mut self, output_shaders: &mut Vec<ShaderCommand>) {
    let cursor = self.frame.cursor();
    let prev_cursor = self.prev_frame.cursor();

    // If cursor is changed.
    if cursor != prev_cursor {
      if cursor.blinking() != prev_cursor.blinking() {
        if cursor.blinking() {
          output_shaders.push(ShaderCommand::CursorEnableBlinking(
            crossterm::cursor::EnableBlinking,
          ));
        } else {
          output_shaders.push(ShaderCommand::CursorDisableBlinking(
            crossterm::cursor::DisableBlinking,
          ));
        }
      }
      if cursor.hidden() != prev_cursor.hidden() {
        if cursor.hidden() {
          output_shaders
            .push(ShaderCommand::CursorHide(crossterm::cursor::Hide));
        } else {
          output_shaders
            .push(ShaderCommand::CursorShow(crossterm::cursor::Show));
        }
      }
      if cursor.style() != prev_cursor.style() {
        output_shaders
          .push(ShaderCommand::CursorSetCursorStyle(cursor.style()));
      }
      if cursor.pos() != prev_cursor.pos() {
        output_shaders.push(ShaderCommand::CursorMoveTo(
          crossterm::cursor::MoveTo(cursor.pos().x(), cursor.pos().y()),
        ));
      }
    }
  }

  /// Shade cells and append results into shader vector.
  pub fn _shade_cells(&mut self, output_shaders: &mut Vec<ShaderCommand>) {
    if self.size() == self.prev_size() {
      // When terminal size doesn't change, use dirty-marks diff-algorithm.
      self._dirty_marks_diff(output_shaders);
    } else {
      // When terminal size changes, use brute-force diff-algorithm.
      self._brute_force_diff(output_shaders)
    }
  }

  pub fn _make_consequential_shaders(
    &self,
    row: u16,
    start_col: u16,
    end_col: u16,
    output_shaders: &mut Vec<ShaderCommand>,
  ) {
    debug_assert!(end_col > start_col);
    let new_cells = self.frame().get_cells_at(
      point!(start_col, row),
      end_col as usize - start_col as usize,
    );
    output_shaders.push(ShaderCommand::CursorMoveTo(
      crossterm::cursor::MoveTo(start_col, row),
    ));

    let get_content = |start_idx, end_idx| {
      let contents = new_cells[start_idx..end_idx]
        .iter()
        .map(|c| {
          if c.symbol().is_empty() {
            " "
          } else {
            c.symbol()
          }
        })
        .join("");
      let fg = new_cells[start_idx].fg();
      let bg = new_cells[start_idx].bg();
      let attrs = new_cells[start_idx].attrs();
      let mut contents = contents.with(*fg).on(*bg);
      if attrs.has(Attribute::Bold) {
        contents = contents.bold();
      }
      if attrs.has(Attribute::Dim) {
        contents = contents.dim();
      }
      if attrs.has(Attribute::Italic) {
        contents = contents.italic();
      }
      if attrs.has(Attribute::Underlined) {
        contents = contents.underlined();
      }
      contents
    };

    let mut start_i: usize = 0;
    for (i, cell) in new_cells.iter().enumerate() {
      let start_cell = &new_cells[start_i];
      if *cell.fg() != *start_cell.fg()
        || *cell.bg() != *start_cell.bg()
        || cell.attrs() != start_cell.attrs()
      {
        let contents = get_content(start_i, i);
        // trace!(
        //   "[{:>2},{:>2}-{:>2},{:>2}-{:>2}], content:{} ({:?})",
        //   row,
        //   start_col,
        //   end_col,
        //   start_i,
        //   i,
        //   contents.content(),
        //   contents.style()
        // );
        output_shaders.push(ShaderCommand::StylePrintStyledContentString(
          crossterm::style::PrintStyledContent(contents),
        ));
        start_i = i;
      }
    }
    if start_i < new_cells.len() {
      let contents = get_content(start_i, new_cells.len());
      // trace!(
      //   "[{:>2},{:>2}-{:>2},{:>2}-{:>2}], content:{} ({:?})",
      //   row,
      //   start_col,
      //   end_col,
      //   start_i,
      //   new_cells.len(),
      //   contents.content(),
      //   contents.style()
      // );
      output_shaders.push(ShaderCommand::StylePrintStyledContentString(
        crossterm::style::PrintStyledContent(contents),
      ));
    }
  }

  /// Brute force diff-algorithm, it iterates all cells on current frame, and compares with
  /// previous frame to find out the changed cells.
  ///
  /// This algorithm is useful when the whole terminal size is changed, and row/column based
  /// diff-algorithm becomes invalid.
  pub fn _brute_force_diff(&mut self, output_shaders: &mut Vec<ShaderCommand>) {
    let size = self.size();
    // trace!("brute force diff, size:{:?}", size);

    if self.frame().is_zero_sized() {
      return;
    }

    for row in 0..size.height() {
      self._make_consequential_shaders(
        row,
        0_u16,
        size.width(),
        output_shaders,
      );
    }
  }

  /// Dirty marks diff-algorithm, it only iterates on the area that has been
  /// marked as dirty by UI widgets.
  ///
  /// This algorithm is more performant when the whole terminal size remains
  /// unchanged.
  pub fn _dirty_marks_diff(&mut self, output_shaders: &mut Vec<ShaderCommand>) {
    let size = self.size();
    // trace!("dirty marks diff, size:{:?}", size);

    if self.frame().is_zero_sized() {
      return;
    }

    let n = (size.height() as usize) * (size.width() as usize);
    let mut start_idx: Option<u32> = None;
    let mut start_pos: Option<U16Pos> = None;
    let mut last_idx: Option<u32> = None;
    let mut last_pos: Option<U16Pos> = None;
    for idx in self.frame().dirty_marks().iter() {
      if (idx as usize) < n {
        let pos = self.frame().idx2pos(idx as usize);
        // trace!(
        //   "dirty idx:{:?},pos:{:?}, start idx:{:?},pos:{:?}",
        //   idx, pos, start_idx, start_pos
        // );
        if start_idx.is_none() && start_pos.is_none() {
          start_idx = Some(idx);
          start_pos = Some(pos);
        } else {
          debug_assert!(start_idx.is_some());
          debug_assert!(start_pos.is_some());
          debug_assert!(last_idx.is_some());
          debug_assert!(last_pos.is_some());

          if let Some(start_pos_unwrap) = start_pos
            && let Some(last_idx) = last_idx
            && let Some(last_pos) = last_pos
          {
            // On the same row and column is consequential.
            if start_pos_unwrap.y() == pos.y()
              && last_pos.y() == pos.y()
              && last_idx + 1 == idx
            {
              // Do nothing and continue iterating dirty marks.
            } else {
              // Render a consequential range of cells.
              if last_pos.x() > start_pos_unwrap.x() {
                self._make_consequential_shaders(
                  start_pos_unwrap.y(),
                  start_pos_unwrap.x(),
                  last_pos.x() + 1,
                  output_shaders,
                );
                start_idx = Some(idx);
                start_pos = Some(pos);
              }
            }
          }
        }
        last_idx = Some(idx);
        last_pos = Some(pos);
      }
    }
    if let Some(start_pos) = start_pos
      && let Some(last_pos) = last_pos
      && last_pos.x() > start_pos.x()
    {
      self._make_consequential_shaders(
        start_pos.y(),
        start_pos.x(),
        last_pos.x() + 1,
        output_shaders,
      );
    }
  }
}
// Shade }

#[derive(Debug, Clone)]
/// Shader command enums.
///
/// All-in-one wrapper to wrap all the [`crossterm::Command`], thus helps to return the rendering
/// updates for the terminal.
pub enum ShaderCommand {
  CursorSetCursorStyle(crossterm::cursor::SetCursorStyle),
  CursorDisableBlinking(crossterm::cursor::DisableBlinking),
  CursorEnableBlinking(crossterm::cursor::EnableBlinking),
  CursorHide(crossterm::cursor::Hide),
  CursorMoveDown(crossterm::cursor::MoveDown),
  CursorMoveLeft(crossterm::cursor::MoveLeft),
  CursorMoveRight(crossterm::cursor::MoveRight),
  CursorMoveTo(crossterm::cursor::MoveTo),
  CursorMoveToColumn(crossterm::cursor::MoveToColumn),
  CursorMoveToNextLine(crossterm::cursor::MoveToNextLine),
  CursorMoveToPreviousLine(crossterm::cursor::MoveToPreviousLine),
  CursorMoveToRow(crossterm::cursor::MoveToRow),
  CursorMoveUp(crossterm::cursor::MoveUp),
  CursorRestorePosition(crossterm::cursor::RestorePosition),
  CursorSavePosition(crossterm::cursor::SavePosition),
  CursorShow(crossterm::cursor::Show),
  EventDisableBracketedPaste(crossterm::event::DisableBracketedPaste),
  EventDisableFocusChange(crossterm::event::DisableFocusChange),
  EventDisableMouseCapture(crossterm::event::DisableMouseCapture),
  EventEnableBracketedPaste(crossterm::event::EnableBracketedPaste),
  EventEnableFocusChange(crossterm::event::EnableFocusChange),
  EventEnableMouseCapture(crossterm::event::EnableMouseCapture),
  EventPopKeyboardEnhancementFlags(
    crossterm::event::PopKeyboardEnhancementFlags,
  ),
  EventPushKeyboardEnhancementFlags(
    crossterm::event::PushKeyboardEnhancementFlags,
  ),
  StyleResetColor(crossterm::style::ResetColor),
  StyleSetAttribute(crossterm::style::SetAttribute),
  StyleSetAttributes(crossterm::style::SetAttributes),
  StyleSetBackgroundColor(crossterm::style::SetBackgroundColor),
  StyleSetColors(crossterm::style::SetColors),
  StyleSetForegroundColor(crossterm::style::SetForegroundColor),
  StyleSetStyle(crossterm::style::SetStyle),
  StyleSetUnderlineColor(crossterm::style::SetUnderlineColor),
  StylePrintStyledContentString(crossterm::style::PrintStyledContent<String>),
  StylePrintString(crossterm::style::Print<String>),
  TerminalBeginSynchronizedUpdate(crossterm::terminal::BeginSynchronizedUpdate),
  TerminalClear(crossterm::terminal::Clear),
  TerminalDisableLineWrap(crossterm::terminal::DisableLineWrap),
  TerminalEnableLineWrap(crossterm::terminal::EnableLineWrap),
  TerminalEndSynchronizedUpdate(crossterm::terminal::EndSynchronizedUpdate),
  TerminalEnterAlternateScreen(crossterm::terminal::EnterAlternateScreen),
  TerminalLeaveAlternateScreen(crossterm::terminal::LeaveAlternateScreen),
  TerminalScrollDown(crossterm::terminal::ScrollDown),
  TerminalScrollUp(crossterm::terminal::ScrollUp),
  TerminalSetSize(crossterm::terminal::SetSize),
}

#[derive(Debug, Clone)]
/// The rendering updates on each draw, returns from [`Canvas::shade`] method.
///
/// It's simply a collection of [`ShaderCommand`].
pub struct Shader {
  commands: Vec<ShaderCommand>,
}

impl Shader {
  /// Make new shader.
  pub fn new(commands: Vec<ShaderCommand>) -> Self {
    Shader { commands }
  }

  /// Get an iterator of the collection.
  pub fn iter(&self) -> Iter<'_, ShaderCommand> {
    self.commands.iter()
  }
}
