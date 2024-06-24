//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]
use crate::geo::pos::UPos;
use crate::geo::size::Size;
use crate::ui::frame::Cursive;
use crate::ui::term::Terminal;
use crate::ui::widget::root::RootWidget;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode,
};
use crossterm::{cursor as termcur, queue, terminal};
use futures::StreamExt;
use std::io::{Result as IoResult, Write};
use tracing::debug;

pub struct EventLoop {
  screen: Terminal,
  root_widget: RootWidget,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let size = Size::new(rows as usize, cols as usize);
    let cursive = Cursive::default();
    let screen = Terminal::new(size, cursive);
    let root_widget = RootWidget::new(size);
    Ok(EventLoop {
      screen,
      root_widget,
    })
  }

  pub async fn init(&self) -> IoResult<()> {
    if !terminal::is_raw_mode_enabled()? {
      terminal::enable_raw_mode()?;
    }

    let mut out = std::io::stdout();
    queue!(
      out,
      terminal::EnterAlternateScreen,
      terminal::Clear(terminal::ClearType::All),
      EnableMouseCapture,
      EnableFocusChange
    )?;

    let cursive = self.screen.cursive();
    if cursive.blinking {
      queue!(out, termcur::EnableBlinking)?;
    } else {
      queue!(out, termcur::DisableBlinking)?;
    }
    if cursive.hidden {
      queue!(out, termcur::Hide)?;
    } else {
      queue!(out, termcur::Show)?;
    }

    queue!(out, cursive.style)?;
    out.flush()?;

    Ok(())
  }

  pub async fn run(&mut self) -> IoResult<()> {
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
      println!("Curosr position: {:?}\r", termcur::position());
    }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue loop
    true
  }

  pub async fn shutdown(&self) -> IoResult<()> {
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
}
