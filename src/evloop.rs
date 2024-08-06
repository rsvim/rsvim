//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode, KeyEventKind, KeyEventState, KeyModifiers,
};
use crossterm::{self, queue, terminal};
use futures::StreamExt;
use geo::point;
use heed::types::U16;
use parking_lot::ReentrantMutexGuard;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::{Result as IoResult, Write};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::geo_size_as;
use crate::glovar;
use crate::ui::frame::CursorStyle;
use crate::ui::term::{Terminal, TerminalArc};
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{
  Cursor, RootContainer, Widget, WidgetValue, WindowContainer, WindowContent,
};

#[derive(Clone, Debug)]
pub struct EventLoop {
  screen: TerminalArc,
  tree: TreeArc,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let screen_size = U16Size::new(cols, rows);
    let screen = Terminal::new(screen_size);
    let screen = Terminal::to_arc(screen);
    let mut tree = Tree::new(screen_size);
    debug!("new, screen size: {:?}", screen_size);

    let window_container = WindowContainer::new();
    let window_container_id = window_container.id();
    let window_container_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    let window_container_node = TreeNode::new(
      WidgetValue::WindowContainer(window_container),
      window_container_shape,
    );
    tree.insert(tree.root_id(), window_container_node);
    debug!("new, insert window container: {:?}", window_container_id);

    let window_content = WindowContent::new();
    let window_content_id = window_content.id();
    let window_content_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    let window_content_node = TreeNode::new(
      WidgetValue::WindowContent(window_content),
      window_content_shape,
    );
    tree.insert(window_container_id, window_content_node);
    debug!("new, insert window content: {:?}", window_content_id);

    let cursor = Cursor::new();
    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor_node = TreeNode::new(WidgetValue::Cursor(cursor), cursor_shape);
    tree.insert(window_content_id, cursor_node);

    debug!("new, built widget tree");

    Ok(EventLoop {
      screen,
      tree: Tree::to_arc(tree),
    })
  }

  pub async fn init(&self) -> IoResult<()> {
    let mut out = std::io::stdout();

    debug!("init, draw cursor");
    let cursor = self.screen.lock().frame().cursor;
    if cursor.blinking {
      queue!(out, crossterm::cursor::EnableBlinking)?;
    } else {
      queue!(out, crossterm::cursor::DisableBlinking)?;
    }
    if cursor.hidden {
      queue!(out, crossterm::cursor::Hide)?;
    } else {
      queue!(out, crossterm::cursor::Show)?;
    }

    queue!(out, cursor.style)?;
    queue!(
      out,
      crossterm::cursor::MoveTo(cursor.pos.x(), cursor.pos.y())
    )?;

    out.flush()?;
    debug!("init, draw cursor - done");

    Ok(())
  }

  pub async fn run(&mut self) -> IoResult<()> {
    let mut reader = EventStream::new();
    loop {
      tokio::select! {
        polled_event = reader.next() => match polled_event {
          Some(Ok(event)) => {
            debug!("run, polled event: {:?}", event);
            match self.accept(event).await {
                Ok(next_loop) => {
                    if !next_loop {
                        break;
                    }
                }
                _ => break
            }
          },
          Some(Err(e)) => {
            debug!("run, error: {:?}", e);
            error!("Error: {:?}\r", e);
            break;
          },
          None => break,
        }
      }
    }
    Ok(())
  }

  pub async fn accept(&mut self, event: Event) -> IoResult<bool> {
    debug!("Event::{:?}", event);
    // println!("Event:{:?}", event);

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {
          match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
              // Up
              let mut tree = self
                .tree
                .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                .unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.move_up_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Down | KeyCode::Char('j') => {
              // Down
              let mut tree = self
                .tree
                .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                .unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.move_down_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Left | KeyCode::Char('h') => {
              // Left
              let mut tree = self
                .tree
                .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                .unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.move_left_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            KeyCode::Right | KeyCode::Char('l') => {
              // Right
              let mut tree = self
                .tree
                .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                .unwrap();
              match tree.cursor_id() {
                Some(cursor_id) => {
                  tree.move_right_by(cursor_id, 1);
                }
                None => { /* Skip */ }
              }
            }
            _ => { /* Skip */ }
          }
        }
        KeyEventKind::Repeat => {}
        KeyEventKind::Release => {}
      },
      Event::Mouse(_mouse_event) => {}
      Event::Paste(ref _paste_string) => {}
      Event::Resize(_columns, _rows) => {}
    }

    // if event == Event::Key(KeyCode::Char('c').into()) {
    //   println!("Curosr position: {:?}\r", crossterm::cursor::position());
    // }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      println!("ESC: {:?}\r", crossterm::cursor::position());
      return Ok(false);
    }

    // Draw UI components to the terminal frame.
    self
      .tree
      .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .draw(self.screen.clone());

    // Flush terminal frame to the device.
    match self
      .screen
      .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .flush()
      .await
    {
      Ok(_) => Ok(true),
      Err(e) => Err(e),
    }
  }
}
