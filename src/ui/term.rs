//! Backend terminal for receiving user inputs & canvas for UI rendering.

use crossterm::{self, queue};
use parking_lot::Mutex;
use std::fmt;
use std::fmt::Debug;
use std::slice::Iter;
use std::sync::Arc;

use crate::cart::U16Size;
use crate::ui::frame::cursor::{cursor_style_eq, CursorStyleFormatter};
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

  /// Get the shader commands that should print to the terminal device.
  /// It uses a diff-algorithm to reduce the output.
  pub fn shade(&mut self) -> Shader {
    let mut shader = Shader::new();

    // For cursor.
    if self.frame.dirty_cursor {
      let cursor = self.frame.cursor;
      let prev_cursor = self.prev_frame.cursor;

      if cursor.blinking != prev_cursor.blinking {
        shader.push(if cursor.blinking {
          ShaderCommand::CursorEnableBlinking(crossterm::cursor::EnableBlinking)
        } else {
          ShaderCommand::CursorDisableBlinking(crossterm::cursor::DisableBlinking)
        });
      }
      if cursor.hidden != prev_cursor.hidden {
        shader.push(if cursor.hidden {
          ShaderCommand::CursorHide(crossterm::cursor::Hide)
        } else {
          ShaderCommand::CursorShow(crossterm::cursor::Show)
        });
      }
      if !cursor_style_eq(cursor.style, prev_cursor.style) {
        shader.push(ShaderCommand::CursorSetCursorStyle(cursor.style));
      }
      if cursor.pos != prev_cursor.pos {
        shader.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
          cursor.pos.x(),
          cursor.pos.y(),
        )));
      }
    }

    // Save current frame.
    self.prev_frame = self.frame.clone();
    // Reset the `dirty` fields.
    self.frame.reset_dirty();

    shader
  }
}

#[derive(Clone)]
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

impl fmt::Debug for ShaderCommand {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    let s = format!("ShaderCommand::{:?}", self);
    f.debug_struct(&s).finish()
  }
}

#[derive(Debug, Default, Clone)]
pub struct Shader {
  commands: Vec<ShaderCommand>,
}

impl Shader {
  pub fn new() -> Self {
    Shader { commands: vec![] }
  }

  pub fn push(&mut self, command: ShaderCommand) {
    self.commands.push(command)
  }

  pub fn iter(&self) -> Iter<ShaderCommand> {
    self.commands.iter()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn new1() {
    let t = Terminal::new(U16Size::new(3, 4));
    assert_eq!(t.frame().size, t.prev_frame().size);
    assert_eq!(t.frame().cursor, t.prev_frame().cursor);
  }

  #[test]
  fn shader_command_debug() {
    assert_eq!(
      format!(
        "{:?}",
        ShaderCommand::TerminalEndSynchronizedUpdate(crossterm::terminal::EndSynchronizedUpdate)
      ),
      "ShaderCommand::TerminalEndSynchronizedUpdate"
    );
  }
}
