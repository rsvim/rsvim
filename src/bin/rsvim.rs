use clap::Parser;
use crossterm::cursor;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use heed::types as heed_types;
use heed::{byteorder, Database, EnvOpenOptions};
use rsvim::{cli, log, ui};
use tracing::debug;

async fn input_loop() -> std::io::Result<()> {
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

  Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let cli = cli::Cli::parse();
  log::init(&cli);
  debug!("cli: {:?}", cli);

  let dir = tempfile::tempdir().unwrap();
  debug!("tempdir:{:?}", dir);
  let env = unsafe { EnvOpenOptions::new().open(dir.path()).unwrap() };
  let mut wtxn = env.write_txn().unwrap();
  let db: Database<heed_types::Str, heed_types::U32<byteorder::NativeEndian>> =
    env.create_database(&mut wtxn, None).unwrap();
  db.put(&mut wtxn, "seven", &7).unwrap();
  wtxn.commit().unwrap();

  ui::device::init().await?;
  input_loop().await?;
  ui::device::shutdown().await
}
