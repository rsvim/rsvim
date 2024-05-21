use crate::geo::size::Size;
use crate::ui::term::Terminal;
use crate::ui::widget::root::widget::RootWidget;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode,
};
use crossterm::{cursor, queue, terminal};
use futures::StreamExt;
use std::io::Write;
use tracing::debug;

pub struct EventLoop {
  screen: Terminal,
  root: RootWidget,
}

impl EventLoop {
  pub fn new() -> std::io::Result<Self> {
    let (cols, rows) = terminal::size()?;
    let size = Size::new(rows as usize, cols as usize);
    let screen = Terminal::new(size);
    let root = RootWidget::new(size);
    Ok(EventLoop { screen, root })
  }

  pub async fn run(&mut self) -> std::io::Result<()> {
    let mut reader = EventStream::new();
    loop {
      tokio::select! {
        polled_event = reader.next() => match polled_event {
          Some(Ok(event)) => {
            if !self.accept(event).await {
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

  pub async fn accept(&mut self, event: Event) -> bool {
    println!("Event::{:?}\r", event);
    debug!("Event::{:?}", event);

    if event == Event::Key(KeyCode::Char('c').into()) {
      println!("Curosr position: {:?}\r", cursor::position());
    }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue loop
    true
  }
}
