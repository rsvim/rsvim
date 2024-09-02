//! Canvas.

use crossterm;
use parking_lot::RwLock;
use std::fmt;
use std::fmt::Debug;
use std::slice::Iter;
use std::sync::Arc;

use crate::cart::{U16Pos, U16Size};

// Re-export
pub use crate::ui::canvas::frame::cell::Cell;
pub use crate::ui::canvas::frame::cursor::{
  cursor_style_eq, Cursor, CursorStyle, CursorStyleFormatter,
};
pub use crate::ui::canvas::frame::Frame;

pub mod frame;

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

pub type CanvasArc = Arc<RwLock<Canvas>>;

impl Canvas {
  /// Make new canvas with terminal actual size.
  pub fn new(size: U16Size) -> Self {
    Canvas {
      prev_frame: Frame::new(size, Cursor::default()),
      frame: Frame::new(size, Cursor::default()),
    }
  }

  /// Convert struct into smart pointer.
  pub fn to_arc(t: Canvas) -> CanvasArc {
    Arc::new(RwLock::new(t))
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
    self.frame.cells()
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
    self.prev_frame.cells()
  }

  /// Get previous frame cells at specific range.
  pub fn prev_cells_at(&self, pos: U16Pos, n: usize) -> &[Cell] {
    self.prev_frame.cells_at(pos, n)
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

    // For cursor
    self._shade_cursor(&mut shader);
    // For cells
    self._shade_cells(&mut shader);

    // Save current frame.
    self.prev_frame = self.frame.clone();
    // Reset the `dirty` fields.
    self.frame.reset_dirty();

    shader
  }

  /// Shade cursor and append results into shader vector.
  pub fn _shade_cursor(&mut self, shader: &mut Shader) {
    // For cursor.
    if self.frame.dirty_cursor() {
      let cursor = self.frame.cursor();
      let prev_cursor = self.prev_frame.cursor();

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
      if !cursor_style_eq(&cursor.style(), &prev_cursor.style()) {
        shader.push(ShaderCommand::CursorSetCursorStyle(cursor.style()));
      }
      if cursor.pos() != prev_cursor.pos() {
        shader.push(ShaderCommand::CursorMoveTo(crossterm::cursor::MoveTo(
          cursor.pos().x(),
          cursor.pos().y(),
        )));
      }
    }
  }

  /// Shade cells and append results into shader vector.
  pub fn _shade_cells(&mut self, shader: &mut Shader) {}
}

#[derive(Clone)]
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
    let s = match self {
      ShaderCommand::CursorSetCursorStyle(command) => format!("{}", command),
      ShaderCommand::CursorDisableBlinking(command) => format!("{:?}", command),
      ShaderCommand::CursorEnableBlinking(command) => format!("{:?}", command),
      ShaderCommand::CursorHide(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveDown(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveLeft(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveRight(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveTo(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveToColumn(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveToNextLine(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveToPreviousLine(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveToRow(command) => format!("{:?}", command),
      ShaderCommand::CursorMoveUp(command) => format!("{:?}", command),
      ShaderCommand::CursorRestorePosition(command) => format!("{:?}", command),
      ShaderCommand::CursorSavePosition(command) => format!("{:?}", command),
      ShaderCommand::CursorShow(command) => format!("{:?}", command),
      ShaderCommand::EventDisableBracketedPaste(command) => format!("{:?}", command),
      ShaderCommand::EventDisableFocusChange(command) => format!("{:?}", command),
      ShaderCommand::EventDisableMouseCapture(command) => format!("{:?}", command),
      ShaderCommand::EventEnableBracketedPaste(command) => format!("{:?}", command),
      ShaderCommand::EventEnableFocusChange(command) => format!("{:?}", command),
      ShaderCommand::EventEnableMouseCapture(command) => format!("{:?}", command),
      ShaderCommand::EventPopKeyboardEnhancementFlags(command) => format!("{:?}", command),
      ShaderCommand::EventPushKeyboardEnhancementFlags(command) => format!("{:?}", command),
      ShaderCommand::StyleResetColor(command) => format!("{:?}", command),
      ShaderCommand::StyleSetAttribute(command) => format!("{:?}", command),
      ShaderCommand::StyleSetAttributes(command) => format!("{:?}", command),
      ShaderCommand::StyleSetBackgroundColor(command) => format!("{:?}", command),
      ShaderCommand::StyleSetColors(command) => format!("{}", command),
      ShaderCommand::StyleSetForegroundColor(command) => format!("{}", command),
      ShaderCommand::StyleSetStyle(command) => format!("{:?}", command),
      ShaderCommand::StyleSetUnderlineColor(command) => format!("{:?}", command),
      ShaderCommand::TerminalBeginSynchronizedUpdate(command) => format!("{:?}", command),
      ShaderCommand::TerminalClear(command) => format!("{}", command),
      ShaderCommand::TerminalDisableLineWrap(command) => format!("{:?}", command),
      ShaderCommand::TerminalEnableLineWrap(command) => format!("{:?}", command),
      ShaderCommand::TerminalEndSynchronizedUpdate(command) => format!("{:?}", command),
      ShaderCommand::TerminalEnterAlternateScreen(command) => format!("{:?}", command),
      ShaderCommand::TerminalLeaveAlternateScreen(command) => format!("{:?}", command),
      ShaderCommand::TerminalScrollDown(command) => format!("{:?}", command),
      ShaderCommand::TerminalScrollUp(command) => format!("{:?}", command),
      ShaderCommand::TerminalSetSize(command) => format!("{:?}", command),
    };
    f.debug_struct(&s).finish()
  }
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

  /// Get an iterator of the collection.
  pub fn iter(&self) -> Iter<ShaderCommand> {
    self.commands.iter()
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Once;
  use tracing::info;

  use crate::test::log::init as test_log_init;

  use super::*;

  static INIT: Once = Once::new();

  #[test]
  fn new1() {
    let t = Canvas::new(U16Size::new(3, 4));
    assert_eq!(t.frame().size(), t.prev_frame().size());
    assert_eq!(*t.frame().cursor(), *t.prev_frame().cursor());
  }

  #[test]
  fn shader_command_debug() {
    INIT.call_once(test_log_init);
    info!(
      "ShaderCommand::TerminalEndSynchronizedUpdate: {:?}",
      ShaderCommand::TerminalEndSynchronizedUpdate(crossterm::terminal::EndSynchronizedUpdate)
    );
    assert_eq!(
      format!(
        "{:?}",
        ShaderCommand::TerminalEndSynchronizedUpdate(crossterm::terminal::EndSynchronizedUpdate)
      ),
      "ShaderCommand::TerminalEndSynchronizedUpdate"
    );
  }
}
