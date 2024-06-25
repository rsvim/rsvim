//! The VIM editor reinvented in Rust+Typescript.

use clap::Parser;
use rsvim::{cli, log};
// use heed::types as heed_types;
// use heed::{byteorder, Database, EnvOpenOptions};
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
};
use crossterm::{execute, terminal};
use futures::StreamExt;
use rsvim::evloop::EventLoop;
use std::io::{Result as IoResult, Write};
use tracing::debug;

pub async fn init() -> IoResult<()> {
  if !terminal::is_raw_mode_enabled()? {
    terminal::enable_raw_mode()?;
  }

  let mut out = std::io::stdout();
  execute!(
    out,
    terminal::EnterAlternateScreen,
    terminal::Clear(terminal::ClearType::All),
    EnableMouseCapture,
    EnableFocusChange,
  )?;

  Ok(())
}

pub async fn shutdown() -> IoResult<()> {
  let mut out = std::io::stdout();
  execute!(
    out,
    DisableMouseCapture,
    DisableFocusChange,
    terminal::LeaveAlternateScreen,
  )?;

  if terminal::is_raw_mode_enabled()? {
    terminal::disable_raw_mode()?;
  }

  Ok(())
}

#[tokio::main]
async fn main() -> IoResult<()> {
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

  init().await?;
  let mut ev = EventLoop::new().await?;
  ev.run().await?;
  shutdown().await
}
