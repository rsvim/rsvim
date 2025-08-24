//! Canvas.

use crate::prelude::*;

// Re-export
pub use frame::cell::*;
pub use frame::cursor::*;
pub use frame::*;

use crossterm;
use geo::point;
use std::fmt::Debug;
use std::slice::Iter;

pub mod frame;
pub mod internal;

#[cfg(test)]
mod frame_tests;

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

  // Previous frame }

  /// Get the shader commands that should print to the terminal device, it internally uses a
  /// diff-algorithm to reduce the outputs.
  pub fn shade(&mut self) -> Shader {
    let mut shader = Shader::new();

    // For cells, it needs extra save and restore cursor position
    let mut cells_shaders = self._shade_cells();
    let saved_cursor_pos = self.cursor().pos();

    // On Windows Terminal, flushing shaders without hiding cursor makes the
    // cursor twinkling/jumping while refreshing the TUI screen.
    // So here let's hide cursor before flushing shaders, and restore the
    // cursor after flushing is done.
    if cfg!(target_os = "windows") && !self.cursor().hidden() {
      shader.push(ShaderCommand::CursorHide(crossterm::cursor::Hide));
    }

    shader.append(&mut cells_shaders);
    shader.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
      saved_cursor_pos.x(),
      saved_cursor_pos.y(),
    )));

    if cfg!(target_os = "windows") && !self.cursor().hidden() {
      shader.push(ShaderCommand::CursorShow(crossterm::cursor::Show));
    }

    // For cursor
    let mut cursor_shaders = self._shade_cursor();
    shader.append(&mut cursor_shaders);

    // Finish shade.
    self._shade_done();

    shader
  }

  /// Shade done.
  pub fn _shade_done(&mut self) {
    // Save current frame.
    self.prev_frame.clone_from(&self.frame);
    // Reset the `dirty` fields.
    self.frame.reset_dirty_rows();
  }

  /// Shade cursor and append results into shader vector.
  pub fn _shade_cursor(&mut self) -> Vec<ShaderCommand> {
    let cursor = self.frame.cursor();
    let prev_cursor = self.prev_frame.cursor();
    let mut shader = vec![];

    // If cursor is changed.
    if cursor != prev_cursor {
      if cursor.blinking() != prev_cursor.blinking() {
        if cursor.blinking() {
          shader.push(ShaderCommand::CursorEnableBlinking(
            crossterm::cursor::EnableBlinking,
          ));
        } else {
          shader.push(ShaderCommand::CursorDisableBlinking(
            crossterm::cursor::DisableBlinking,
          ));
        }
      }
      if cursor.hidden() != prev_cursor.hidden() {
        if cursor.hidden() {
          shader.push(ShaderCommand::CursorHide(crossterm::cursor::Hide));
        } else {
          shader.push(ShaderCommand::CursorShow(crossterm::cursor::Show));
        }
      }
      if cursor.style() != prev_cursor.style() {
        shader.push(ShaderCommand::CursorSetCursorStyle(cursor.style()));
      }
      if cursor.pos() != prev_cursor.pos() {
        shader.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
          cursor.pos().x(),
          cursor.pos().y(),
        )));
      }
    }

    shader
  }

  /// Shade cells and append results into shader vector.
  pub fn _shade_cells(&mut self) -> Vec<ShaderCommand> {
    if self.size() == self.prev_size() {
      // When terminal size remains the same, use dirty-marks diff-algorithm.
      self._dirty_marks_diff()
    } else {
      // When terminal size remains the same, use brute-force diff-algorithm.
      self._brute_force_diff()
    }
  }

  /// Find next same cell in current row of frame. NOTE: row is y, col is x.
  ///
  /// # Returns
  ///
  /// 1. The column number if found the same cell, column number started from 0.
  /// 2. The end column index on the row if not found, i.e. the width of current frame.
  pub fn _next_same_cell_in_row(&self, row: u16, col: u16) -> u16 {
    let frame = self.frame();
    let prev_frame = self.prev_frame();

    let mut col_end_at = col;
    while col_end_at < frame.size().width() {
      let pos: U16Pos = point!(x: col_end_at, y: row);
      let cell2 = frame.get_cell(pos);
      let prev_cell2 = prev_frame.get_cell(pos);
      if cell2 == prev_cell2 {
        break;
      }
      col_end_at += 1;
    }
    col_end_at
  }

  pub fn _make_printable_shaders(
    &self,
    row: u16,
    start_col: u16,
    end_col: u16,
  ) -> Vec<ShaderCommand> {
    let frame = self.frame();
    let mut shaders = Vec::new();

    debug_assert!(end_col > start_col);
    let new_cells = frame.get_cells_at(
      point!(x: start_col, y: row),
      end_col as usize - start_col as usize,
    );
    let new_contents = new_cells
      .iter()
      .map(|c| c.symbol().clone())
      .collect::<Vec<_>>()
      .join("");
    shaders.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
      start_col, row,
    )));
    shaders.push(ShaderCommand::StylePrintString(crossterm::style::Print(
      new_contents.to_string(),
    )));
    shaders
  }

  /// Brute force diff-algorithm, it iterates all cells on current frame, and compares with
  /// previous frame to find out the changed cells.
  ///
  /// This algorithm is useful when the whole terminal size is changed, and row/column based
  /// diff-algorithm becomes invalid.
  pub fn _brute_force_diff(&mut self) -> Vec<ShaderCommand> {
    let frame = self.frame();
    let size = self.size();
    let prev_frame = self.prev_frame();
    let _prev_size = self.prev_size();
    trace!("brute force diff, size:{:?}", size);

    let mut shaders = vec![];

    if !frame.zero_sized() {
      for row in 0..size.height() {
        let mut col = 0_u16;
        while col < size.width() {
          // Skip unchanged columns
          let pos: U16Pos = point!(x: col, y: row);
          let cell = frame.get_cell(pos);
          let prev_cell = prev_frame.get_cell(pos);
          if cell == prev_cell {
            col += 1;
            continue;
          }

          // Find the continuously changed parts by iterating over columns
          let col_end_at = self._next_same_cell_in_row(row, col);

          if col_end_at > col {
            let mut print_shaders =
              self._make_printable_shaders(row, col, col_end_at);
            shaders.append(&mut print_shaders);
            col = col_end_at;
          }
        }
      }
    }

    shaders
  }

  /// Dirty marks diff-algorithm, it only iterates on the area that has been marked as dirty by UI
  /// widgets.
  ///
  /// This algorithm is more performant when the whole terminal size remains unchanged.
  pub fn _dirty_marks_diff(&mut self) -> Vec<ShaderCommand> {
    let frame = self.frame();
    let size = self.size();
    let prev_frame = self.prev_frame();
    let _prev_size = self.prev_size();
    trace!("dirty marks diff, size:{:?}", size);

    let mut shaders = vec![];

    if !frame.zero_sized() {
      for (row, dirty) in frame.dirty_rows().iter().enumerate() {
        if row < size.height() as usize && *dirty {
          let mut col = 0_u16;
          while col < size.width() {
            // Skip unchanged columns
            let pos: U16Pos = point!(x: col, y: row as u16);
            let cell = frame.get_cell(pos);
            let prev_cell = prev_frame.get_cell(pos);
            if cell == prev_cell {
              col += 1;
              continue;
            }

            // Find the continuously changed parts by iterating over columns
            let col_end_at = self._next_same_cell_in_row(row as u16, col);

            if col_end_at > col {
              let mut print_shaders =
                self._make_printable_shaders(row as u16, col, col_end_at);
              shaders.append(&mut print_shaders);
              col = col_end_at;
            }
          }
        }
      }
    }

    shaders
  }
}

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

#[derive(Debug, Default, Clone)]
/// The rendering updates on each draw, returns from [`Canvas::shade`] method.
///
/// It's simply a collection of [`ShaderCommand`].
pub struct Shader {
  commands: Vec<ShaderCommand>,
}

impl Shader {
  /// Make new shader.
  pub fn new() -> Self {
    Shader { commands: vec![] }
  }

  /// Push a shader command.
  pub fn push(&mut self, command: ShaderCommand) {
    self.commands.push(command)
  }

  /// Append a vector of shader commands.
  pub fn append(&mut self, commands: &mut Vec<ShaderCommand>) {
    self.commands.append(commands);
  }

  /// Get an iterator of the collection.
  pub fn iter(&self) -> Iter<'_, ShaderCommand> {
    self.commands.iter()
  }
}
