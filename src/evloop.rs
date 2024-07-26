//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]
use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::geo_size_as;
use crate::ui::frame::CursorStyle;
use crate::ui::term::{make_terminal_ptr, Terminal, TerminalPtr};
use crate::ui::tree::node::{make_node_ptr, Node, NodeId};
use crate::ui::tree::{make_tree_ptr, Tree, TreePtr};
use crate::ui::widget::Cursor;
use crate::ui::widget::RootLayout;
use crate::ui::widget::Widget;
use crate::ui::widget::Window;
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode,
};
use crossterm::{cursor as termcursor, queue, terminal};
use futures::StreamExt;
use geo::point;
use heed::types::U16;
use std::borrow::Borrow;
use std::io::{Result as IoResult, Write};
use std::sync::{Arc, RwLock};
use tracing::debug;

pub struct EventLoop {
  screen: TerminalPtr,
  tree: TreePtr,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let screen_size = U16Size::new(cols, rows);
    let screen = Terminal::new(screen_size);
    let screen = make_terminal_ptr(screen);
    let mut tree = Tree::new(Arc::downgrade(&screen));

    let root_widget = RootLayout::default();
    let root_widget_id = root_widget.id();
    let root_widget_node = make_node_ptr(Node::RootLayout(root_widget));
    tree.insert_node(
      root_widget_id,
      root_widget_node.clone(),
      None,
      IRect::new(
        (0, 0),
        (screen_size.width() as isize, screen_size.height() as isize),
      ),
    );

    let window = Window::default();
    let window_id = window.id();
    let window_node = make_node_ptr(Node::Window(window));
    let window_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    tree.insert_node(
      window_id,
      window_node.clone(),
      Some(root_widget_id),
      window_shape,
    );

    let cursor = Cursor::default();
    let cursor_id = cursor.id();
    let cursor_node = make_node_ptr(Node::Cursor(cursor));
    let cursor_shape = IRect::new((0, 0), (1, 1));
    tree.insert_node(
      cursor_id,
      cursor_node.clone(),
      Some(window_id),
      cursor_shape,
    );

    Ok(EventLoop {
      screen,
      tree: make_tree_ptr(tree),
    })
  }

  pub async fn init(&self) -> IoResult<()> {
    let mut out = std::io::stdout();

    let cursor = self.screen.read().unwrap().frame().cursor;
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
