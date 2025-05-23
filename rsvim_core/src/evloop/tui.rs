//! TUI utility.

use crate::prelude::*;

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{self, execute};

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
