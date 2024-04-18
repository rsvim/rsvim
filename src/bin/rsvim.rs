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
use rsvim::{cli, dvc, log};
use std::io::stdout;
use std::time::Duration;
use std::{fs, path, thread, time};
use tracing::{debug, error};

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

  dvc::init().await?;
  input_loop().await?;
  dvc::shutdown().await
}
