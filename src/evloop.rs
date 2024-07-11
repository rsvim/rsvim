//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]
use crate::geom::{IRect, U16Rect, URect};
use crate::ui::term::Terminal;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootWidget;
use crate::ui::widget::window::Window;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode,
};
use crossterm::{cursor as termcursor, queue, terminal};
use futures::StreamExt;
use geo::point;
use heed::types::U16;
use std::io::{Result as IoResult, Write};
use std::sync::{Arc, RwLock};
use tracing::debug;

pub struct EventLoop {
  screen: Terminal,
  root_widget: RootWidget,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let size = U16Rect::new((0_u16, 0_u16), (cols, rows));
    let screen = Terminal::new(size, Default::default());
    let root_widget = RootWidget::new(USize::new(size.height as usize, size.width as usize));

    let cursor_rect = IRect::new(point! (x:0, y:0), point! (x:1 , y:1));
    let cursor_widget = Cursor::new(RootWidget::to_widget_arc(root_widget), cursor_rect);
    Ok(EventLoop {
      screen,
      root_widget,
    })
  }

  pub async fn init(&self) -> IoResult<()> {
    let mut out = std::io::stdout();

    let cursor = self.screen.frame().cursor;
    if cursor.blinking {
      queue!(out, termcursor::EnableBlinking)?;
    } else {
      queue!(out, termcursor::DisableBlinking)?;
    }
    if cursor.hidden {
      queue!(out, termcursor::Hide)?;
    } else {
      queue!(out, termcursor::Show)?;
    }

    queue!(out, cursor.style)?;
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
      println!("Curosr position: {:?}\r", termcursor::position());
    }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue loop
    true
  }
}
