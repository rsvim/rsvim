use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;

pub async fn init() -> std::io::Result<()> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;

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

  Ok(())
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
