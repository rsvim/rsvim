//! Editor mode writer.

use crate::evloop::writer::StdoutWriter;
use crate::evloop::writer::tui;
use crate::prelude::*;
use crate::ui::canvas::{Canvas, Shader, ShaderCommand};

use crossterm::queue;
use std::io::{BufWriter, Stdout, Write};

#[derive(Debug)]
/// Editor mode writer, it writes the canvas to terminal.
pub struct EditorWriter {
  out: BufWriter<Stdout>,
}

impl EditorWriter {
  pub fn new() -> Self {
    Self {
      out: BufWriter::new(std::io::stdout()),
    }
  }
}

impl StdoutWriter for EditorWriter {
  /// Initialize TUI, i.e. enter terminal raw mode.
  fn init(&self) -> IoResult<()> {
    tui::initialize_raw_mode()?;

    // Register panic hook to shutdown terminal raw mode, this helps recover normal terminal
    // command line for users, if any exceptions been thrown.
    tui::shutdown_raw_mode_on_panic();

    Ok(())
  }

  /// Initialize TUI completely, i.e. first flush canvas to terminal.
  fn init_complete(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    // Initialize cursor
    let cursor = canvas.frame().cursor();

    if cursor.blinking() {
      queue!(self.out, crossterm::cursor::EnableBlinking)?;
    } else {
      queue!(self.out, crossterm::cursor::DisableBlinking)?;
    }
    if cursor.hidden() {
      queue!(self.out, crossterm::cursor::Hide)?;
    } else {
      queue!(self.out, crossterm::cursor::Show)?;
    }

    queue!(self.out, cursor.style())?;
    queue!(
      self.out,
      crossterm::cursor::MoveTo(cursor.pos().x(), cursor.pos().y())
    )?;

    self.write(canvas)?;

    Ok(())
  }

  /// Shutdown TUI, i.e. exit terminal raw mode.
  fn shutdown(&self) -> IoResult<()> {
    tui::shutdown_raw_mode()
  }

  /// Write canvas to terminal through STDOUT.
  fn write(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    // Compute the commands that need to output to the terminal device.
    let shader = canvas.shade();
    self.dispatch_shader(shader)?;
    self.out.flush()?;

    Ok(())
  }
}

impl EditorWriter {
  /// Render (queue) shader.
  fn dispatch_shader(&mut self, shader: Shader) -> IoResult<()> {
    for shader_command in shader.iter() {
      match shader_command {
        ShaderCommand::CursorSetCursorStyle(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorDisableBlinking(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorEnableBlinking(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorHide(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveDown(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveLeft(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveRight(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveTo(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveToColumn(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorMoveToNextLine(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorMoveToPreviousLine(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorMoveToRow(command) => queue!(self.out, command)?,
        ShaderCommand::CursorMoveUp(command) => queue!(self.out, command)?,
        ShaderCommand::CursorRestorePosition(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorSavePosition(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::CursorShow(command) => queue!(self.out, command)?,
        ShaderCommand::EventDisableBracketedPaste(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventDisableFocusChange(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventDisableMouseCapture(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventEnableBracketedPaste(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventEnableFocusChange(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventEnableMouseCapture(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventPopKeyboardEnhancementFlags(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::EventPushKeyboardEnhancementFlags(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StyleResetColor(command) => queue!(self.out, command)?,
        ShaderCommand::StyleSetAttribute(command) => queue!(self.out, command)?,
        ShaderCommand::StyleSetAttributes(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StyleSetBackgroundColor(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StyleSetColors(command) => queue!(self.out, command)?,
        ShaderCommand::StyleSetForegroundColor(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StyleSetStyle(command) => queue!(self.out, command)?,
        ShaderCommand::StyleSetUnderlineColor(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StylePrintStyledContentString(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::StylePrintString(command) => queue!(self.out, command)?,
        ShaderCommand::TerminalBeginSynchronizedUpdate(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalClear(command) => queue!(self.out, command)?,
        ShaderCommand::TerminalDisableLineWrap(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalEnableLineWrap(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalEndSynchronizedUpdate(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalEnterAlternateScreen(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalLeaveAlternateScreen(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalScrollDown(command) => {
          queue!(self.out, command)?
        }
        ShaderCommand::TerminalScrollUp(command) => queue!(self.out, command)?,
        ShaderCommand::TerminalSetSize(command) => queue!(self.out, command)?,
      }
    }

    Ok(())
  }
}

impl Default for EditorWriter {
  fn default() -> Self {
    Self::new()
  }
}
