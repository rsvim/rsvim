use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;

pub fn hello() -> std::io::Result<()> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }
  let (cols, rows) = terminal::size()?;

  execute!(
    stdout(),
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBar,
    cursor::Show,
    SetForegroundColor(Color::Yellow),
    SetBackgroundColor(Color::DarkGrey),
    Print(format!(
      "Hello Rsvim! This is a {cols} rows, {rows} columns terminal!"
    )),
    ResetColor,
    terminal::LeaveAlternateScreen
  )?;

  Ok(())
}

fn main() {
  let _ = hello();
}
