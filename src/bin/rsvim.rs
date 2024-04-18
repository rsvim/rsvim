#![allow(unused_imports)]

use clap::Parser;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode,
};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::{cursor, execute, terminal};
use futures::StreamExt;
use heed::{self, Database, EnvOpenOptions};
use rsvim::cli::Cli;
use rsvim::log;
use std::io::stdout;
use std::time::Duration;
use std::{fs, path, thread, time};
use tracing::{debug, error};

async fn input_loop() -> std::io::Result<()> {
  terminal::enable_raw_mode()?;
  let (cols, rows) = terminal::size()?;

  execute!(stdout(), EnableMouseCapture)?;
  execute!(stdout(), EnableFocusChange)?;

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

  let mut reader = EventStream::new();
  loop {
    tokio::select! {
      event_result = reader.next() => match event_result {
        Some(Ok(event)) => {
          println!("Event::{:?}\r", event);
          debug!("Event::{:?}", event);

          if event == Event::Key(KeyCode::Char('c').into()) {
            println!("Curosr position: {:?}\r", cursor::position());
          }

          if event == Event::Key(KeyCode::Esc.into()) {
            break;
          }
        }
        Some(Err(e)) => println!("Error: {:?}\r", e),
        None => break,
      }
    }
  }

  execute!(stdout(), terminal::LeaveAlternateScreen)?;

  execute!(stdout(), DisableMouseCapture)?;
  execute!(stdout(), DisableFocusChange)?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  println!("{}", msg);
  Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let cli = Cli::parse();
  log::init(&cli);
  debug!("cli: {:?}", cli);
  input_loop().await
}
