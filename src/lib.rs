use crossterm::{cursor, execute, terminal};
use std::io::stdout;

use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};

pub fn example() -> std::io::Result<()> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }
  let (cols, rows) = terminal::size()?;

  execute!(
    stdout(),
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBar,
    cursor::Show,
    SetForegroundColor(Color::Yellow),
    SetBackgroundColor(Color::DarkGrey),
    Print(format!(
      "Hello Rsvim! This is a {cols} rows, {rows} columns terminal!"
    )),
    ResetColor
  )?;

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_example() {
    let actual = example();
    assert!(actual.is_ok());
  }
}
