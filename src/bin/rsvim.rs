use clap::Parser;
use crossterm::cursor;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
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

  ui::device::init().await?;
  input_loop().await?;
  ui::device::shutdown().await
}
