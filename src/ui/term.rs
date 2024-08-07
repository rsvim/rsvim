//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crossterm::{self, queue};
use parking_lot::Mutex;
use std::fmt::{Debug, Display};
use std::sync::Arc;

use crate::cart::U16Size;
use crate::ui::frame::{Cell, Cursor, Frame};

/// Backend terminal
#[derive(Debug, Clone)]
pub struct Terminal {
  frame: Frame,
  prev_frame: Frame,
}

pub type TerminalArc = Arc<Mutex<Terminal>>;

impl Terminal {
  pub fn new(size: U16Size) -> Self {
    Terminal {
      prev_frame: Frame::new(size, Cursor::default()),
      frame: Frame::new(size, Cursor::default()),
    }
  }

  pub fn to_arc(t: Terminal) -> TerminalArc {
    Arc::new(Mutex::new(t))
  }

  // Current frame {

  pub fn frame(&self) -> &Frame {
    &self.frame
  }

  pub fn frame_mut(&mut self) -> &mut Frame {
    &mut self.frame
  }

  pub fn size(&self) -> U16Size {
    self.frame.size
  }

  pub fn set_size(&mut self, size: U16Size) {
    self.frame.size = size;
  }

  pub fn cells(&self) -> &Vec<Cell> {
    &self.frame.cells
  }

  pub fn cells_mut(&mut self) -> &mut Vec<Cell> {
    &mut self.frame.cells
  }

  pub fn cursor(&self) -> &Cursor {
    &self.frame.cursor
  }

  pub fn cursor_mut(&mut self) -> &mut Cursor {
    &mut self.frame.cursor
  }

  // Current frame }

  // Previous frame {

  pub fn prev_frame(&self) -> &Frame {
    &self.prev_frame
  }

  pub fn prev_size(&self) -> U16Size {
    self.prev_frame.size
  }

  pub fn prev_cells(&self) -> &Vec<Cell> {
    &self.prev_frame.cells
  }

  pub fn prev_cursor(&self) -> &Cursor {
    &self.prev_frame.cursor
  }

  // Previous frame }

  pub fn compare(&mut self) {
    // Dump current frame to device, with a diff-algorithm to reduce the output.
    if self.frame.dirty_cursor {
      let mut out = std::io::stdout();

      let cursor = self.frame.cursor;
      if cursor.blinking {
        queue!(out, crossterm::cursor::EnableBlinking)?;
      } else {
        queue!(out, crossterm::cursor::DisableBlinking)?;
      }
      if cursor.hidden {
        queue!(out, crossterm::cursor::Hide)?;
      } else {
        queue!(out, crossterm::cursor::Show)?;
      }
      queue!(out, cursor.style)?;
      queue!(
        out,
        crossterm::cursor::MoveTo(cursor.pos.x(), cursor.pos.y())
      )?;
    }

    // Save current frame.
    self.prev_frame = self.frame.clone();
    // Reset the `dirty` fields.
    self.frame.reset_dirty();
  }
}

/// Generic parameter `C` is for `crossterm::style::PrintStyledContent<C>`.
/// Generic parameter `P` is for `crossterm::style::Print<P>`.
/// Generic parameter `T` is for `crossterm::terminal::SetTitle<T>`.
pub enum CrosstermCommand<C, P, T>
where
  C: Display + Debug + Clone + Copy,
  P: Display + Debug + Clone + Copy + PartialEq + Eq,
  T: Display + Debug + Clone + Copy + PartialEq + Eq,
{
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
  EventPopKeyboardEnhancementFlags(crossterm::event::PopKeyboardEnhancementFlags),
  EventPushKeyboardEnhancementFlags(crossterm::event::PushKeyboardEnhancementFlags),
  StyleResetColor(crossterm::style::ResetColor),
  StyleSetAttribute(crossterm::style::SetAttribute),
  StyleSetAttributes(crossterm::style::SetAttributes),
  StyleSetBackgroundColor(crossterm::style::SetBackgroundColor),
  StyleSetColors(crossterm::style::SetColors),
  StyleSetForegroundColor(crossterm::style::SetForegroundColor),
  StyleSetStyle(crossterm::style::SetStyle),
  StyleSetUnderlineColor(crossterm::style::SetUnderlineColor),
  StylePrintStyledContent(crossterm::style::PrintStyledContent<C>),
  StylePrint(crossterm::style::Print<P>),
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
  TerminalSetTitle(crossterm::terminal::SetTitle<T>),
}

pub struct CrosstermCommands {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new1() {
    let t = Terminal::new(U16Size::new(3, 4));
    assert_eq!(t.frame().size, t.prev_frame().size);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }
}
