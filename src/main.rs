use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;
use std::{thread, time};

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
  )?;

  let mut i = 1;
  let timeout = 3;
  loop {
    thread::sleep(time::Duration::from_secs(1));
    i += 1;
    if i > timeout {
      break;
    }
  }

  execute!(stdout(), terminal::LeaveAlternateScreen)?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

fn main() {
  let _ = hello();
}
