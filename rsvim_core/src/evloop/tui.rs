//! TUI utility.

use crate::prelude::*;
use crate::ui::canvas::{Canvas, Shader, ShaderCommand};

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange,
  EnableMouseCapture,
};
use crossterm::{execute, queue};
use std::io::{BufWriter, Stdout, Write};

/// Initialize terminal raw mode.
pub fn initialize_raw_mode() -> IoResult<()> {
  if !crossterm::terminal::is_raw_mode_enabled()? {
    crossterm::terminal::enable_raw_mode()?;
  }

  let mut out = std::io::stdout();
  execute!(
    out,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
    EnableMouseCapture,
    EnableFocusChange,
  )?;
  Ok(())
}

/// Shutdown terminal raw mode.
pub fn shutdown_raw_mode() -> IoResult<()> {
  let mut out = std::io::stdout();
  execute!(
    out,
    DisableMouseCapture,
    DisableFocusChange,
    crossterm::terminal::LeaveAlternateScreen,
  )?;

  if crossterm::terminal::is_raw_mode_enabled()? {
    crossterm::terminal::disable_raw_mode()?;
  }

  Ok(())
}

/// Shutdown terminal raw mode when panic, and dump backtrace.
pub fn shutdown_raw_mode_on_panic() {
  std::panic::set_hook(Box::new(|panic_hook_info| {
    // Recover terminal mode.
    if shutdown_raw_mode().is_err() {
      eprintln!("FATAL! Failed to recover terminal!");
    }

    let now = jiff::Zoned::now();
    let btrace = std::backtrace::Backtrace::force_capture();
    println!("FATAL! Rsvim panics at {now}!");
    println!("{panic_hook_info:?}");
    println!("{btrace}");
    let log_name = format!(
      "rsvim_coredump_{:0>4}-{:0>2}-{:0>2}_{:0>2}-{:0>2}-{:0>2}-{:0>3}.log",
      now.date().year(),
      now.date().month(),
      now.date().day(),
      now.time().hour(),
      now.time().minute(),
      now.time().second(),
      now.time().millisecond(),
    );
    let log_path = std::path::Path::new(log_name.as_str());
    if let Ok(mut f) = std::fs::File::create(log_path) {
      if f
        .write_all(
          format!("FATAL! Rsvim panics!\n{panic_hook_info:?}\n{btrace}")
            .as_bytes(),
        )
        .is_err()
      {
        eprintln!("FATAL! Failed to write rsvim coredump!");
      }
    } else {
      eprintln!("FATAL! Failed to create rsvim coredump!");
    }
  }));
}

#[derive(Debug)]
/// Editor mode writer, terminal raw mode with TUI.
pub struct EditorModeWriter {
  /// Stdout writer for UI.
  pub out: BufWriter<Stdout>,
}

impl EditorModeWriter {
  pub fn new() -> Self {
    Self {
      out: BufWriter::new(std::io::stdout()),
    }
  }

  /// Initialize TUI, i.e. enter terminal raw mode.
  pub fn init_tui(&self) -> IoResult<()> {
    initialize_raw_mode()?;

    // Register panic hook to shutdown terminal raw mode, this helps recover normal terminal
    // command line for users, if any exceptions been thrown.
    shutdown_raw_mode_on_panic();

    Ok(())
  }

  /// Initialize TUI completely, i.e. first flush canvas to terminal.
  pub fn init_tui_complete(&mut self, canvas: &mut Canvas) -> IoResult<()> {
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
  pub fn shutdown_tui(&self) -> IoResult<()> {
    shutdown_raw_mode()
  }

  /// Write canvas to terminal through STDOUT.
  pub fn write(&mut self, canvas: &mut Canvas) -> IoResult<()> {
    // Compute the commands that need to output to the terminal device.
    let shader = canvas.shade();
    self.dispatch_shader(shader)?;
    self.out.flush()?;

    Ok(())
  }

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

impl Default for EditorModeWriter {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
/// Headless mode writer, terminal normal mode without TUI.
pub struct HeadlessModeWriter {}
