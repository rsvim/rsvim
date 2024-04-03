use clap::Parser;
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, execute, terminal};
use rsvim::cli::Cli;
use std::io::stdout;
use std::{thread, time};
use tracing::{self, debug, info};
use tracing_appender;
use tracing_subscriber;

pub fn hello() -> std::io::Result<()> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }
  let (cols, rows) = terminal::size()?;

  let msg = format!("Hello Rsvim! This is a {rows} row, {cols} column terminal!");
  execute!(
    stdout(),
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBar,
    cursor::Show,
    cursor::MoveTo(cols / 2 - (msg.len() / 2) as u16, rows / 2),
    SetForegroundColor(Color::Yellow),
    SetBackgroundColor(Color::DarkGrey),
    Print(&msg),
    ResetColor,
  )?;

  let corners: Vec<(u16, u16)> = vec![(0, 0), (cols, 0), (0, rows), (cols, rows)];
  for corner in corners {
    let msg = format!("Here's column:{}, row:{}!", corner.0, corner.1);
    let (mut c, r) = corner;
    if c > 0 {
      c = c - (msg.len() as u16);
    }
    execute!(
      stdout(),
      cursor::MoveTo(c, r),
      SetForegroundColor(Color::Yellow),
      SetBackgroundColor(Color::DarkGrey),
      Print(msg),
      ResetColor,
    )?;
    thread::sleep(time::Duration::from_secs(1));
  }

  execute!(stdout(), terminal::LeaveAlternateScreen)?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  println!("{}", msg);
  Ok(())
}

fn main() {
  let cli = Cli::parse();
  let file_appender = tracing_appender::rolling::daily("", "rsvim.log");
  let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_file(true)
    .with_line_number(true)
    .with_thread_ids(true)
    .with_thread_names(true)
    .with_level(true)
    .with_writer(non_blocking_appender)
    .finish();
  info!("cli: {:?}", cli);
  // let _ = hello();
}
