pub mod state;

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, terminal};
use state::State;
use std::io::stdout;

pub async fn init() -> std::io::Result<State> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;
  let stat = State::new(cols, rows);

  execute!(std::io::stdout(), EnableMouseCapture)?;
  execute!(std::io::stdout(), EnableFocusChange)?;

  execute!(
    stdout(),
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::Show,
    cursor::MoveTo(0, 0),
  )?;

  Ok(stat)
}

pub async fn shutdown() -> std::io::Result<()> {
  execute!(
    stdout(),
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}
