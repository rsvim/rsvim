//! Editor mode writer.

use crate::evloop::writer::StdoutWritable;
use crate::evloop::writer::tui;
use crate::prelude::*;
use crate::ui::canvas::{Canvas, Shader, ShaderCommand};

use crossterm::queue;
use std::io::{BufWriter, Stdout, Write};

#[derive(Debug)]
/// Editor mode writer, i.e. it writes the canvas to terminal.
pub struct EditorWriter {
  output: BufWriter<Stdout>,
}

impl EditorWriter {
  pub fn new() -> Self {
    Self {
      output: BufWriter::new(std::io::stdout()),
    }
  }
}

impl StdoutWritable for EditorWriter {
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
      queue!(self.output, crossterm::cursor::EnableBlinking)?;
    } else {
      queue!(self.output, crossterm::cursor::DisableBlinking)?;
    }
    if cursor.hidden() {
      queue!(self.output, crossterm::cursor::Hide)?;
    } else {
      queue!(self.output, crossterm::cursor::Show)?;
    }

    queue!(self.output, cursor.style())?;
    queue!(
      self.output,
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
    self.output.flush()?;

    Ok(())
  }
}

impl EditorWriter {
  /// Render (queue) shader.
  fn dispatch_shader(&mut self, shader: Shader) -> IoResult<()> {
    for shader_command in shader.iter() {
      match shader_command {
        ShaderCommand::CursorSetCursorStyle(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorDisableBlinking(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorEnableBlinking(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorHide(command) => queue!(self.output, command)?,
        ShaderCommand::CursorMoveDown(command) => queue!(self.output, command)?,
        ShaderCommand::CursorMoveLeft(command) => queue!(self.output, command)?,
        ShaderCommand::CursorMoveRight(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorMoveTo(command) => queue!(self.output, command)?,
        ShaderCommand::CursorMoveToColumn(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorMoveToNextLine(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorMoveToPreviousLine(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorMoveToRow(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorMoveUp(command) => queue!(self.output, command)?,
        ShaderCommand::CursorRestorePosition(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorSavePosition(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::CursorShow(command) => queue!(self.output, command)?,
        ShaderCommand::EventDisableBracketedPaste(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventDisableFocusChange(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventDisableMouseCapture(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventEnableBracketedPaste(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventEnableFocusChange(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventEnableMouseCapture(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventPopKeyboardEnhancementFlags(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::EventPushKeyboardEnhancementFlags(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleResetColor(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleSetAttribute(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleSetAttributes(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleSetBackgroundColor(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleSetColors(command) => queue!(self.output, command)?,
        ShaderCommand::StyleSetForegroundColor(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StyleSetStyle(command) => queue!(self.output, command)?,
        ShaderCommand::StyleSetUnderlineColor(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StylePrintStyledContentString(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::StylePrintString(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalBeginSynchronizedUpdate(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalClear(command) => queue!(self.output, command)?,
        ShaderCommand::TerminalDisableLineWrap(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalEnableLineWrap(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalEndSynchronizedUpdate(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalEnterAlternateScreen(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalLeaveAlternateScreen(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalScrollDown(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalScrollUp(command) => {
          queue!(self.output, command)?
        }
        ShaderCommand::TerminalSetSize(command) => {
          queue!(self.output, command)?
        }
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
