//! The VIM editor reinvented in Rust+Typescript.

use clap::Parser;
use rsvim::{cli, log, ui};
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use crate::ui::geo::size::Size;
use crate::ui::term::Terminal;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, EventStream,
};
use crossterm::{cursor, queue, terminal};
use futures::StreamExt;
use std::io::Write;
use tracing::debug;

pub async fn init() -> std::io::Result<Terminal> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }

  let (cols, rows) = terminal::size()?;
  let size = Size::new(rows as usize, cols as usize);
  let mut out = std::io::stdout();

  queue!(out, EnableMouseCapture)?;
  queue!(out, EnableFocusChange)?;

  queue!(
    out,
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    cursor::SetCursorStyle::BlinkingBlock,
    cursor::MoveTo(0, 0),
    cursor::Show,
  )?;

  out.flush()?;

  let t = Terminal::new(size);
  Ok(t)
}

pub async fn shutdown() -> std::io::Result<()> {
  let mut out = std::io::stdout();
  queue!(
    out,
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  out.flush()?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

pub async fn run(t: &mut Terminal) -> std::io::Result<()> {
  let mut reader = EventStream::new();
  loop {
    tokio::select! {
      polled_event = reader.next() => match polled_event {
        Some(Ok(event)) => {
          if !t.accept(event).await {
              break;
          }
        },
        Some(Err(e)) => {
          println!("Error: {:?}\r", e);
          break;
        },
        None => break,
      }
    }
  }
  Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let cli = cli::Cli::parse();
  log::init(&cli);
  debug!("cli: {:?}", cli);

  // let dir = tempfile::tempdir().unwrap();
  // debug!("tempdir:{:?}", dir);
  // let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  // let mut wtxn = env.write_txn().unwrap();
  // let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
  //   env.create_database(&mut wtxn, None).unwrap();
  // db.put(&mut wtxn, "seven", &7).unwrap();
  // wtxn.commit().unwrap();

  let mut t = init().await?;
  run(&mut t).await?;
  shutdown().await
}
