//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]
use crate::cart::{IRect, U16Rect, U16Size, URect};
use crate::ui::term::{make_terminal_ptr, Terminal, TerminalPtr};
use crate::ui::tree::node::{make_node_ptr, Node, NodeId};
use crate::ui::tree::{make_tree_ptr, Tree, TreePtr};
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::root::RootWidget;
use crate::ui::widget::window::Window;
use crate::ui::widget::Widget;
use crossterm::cursor::SetCursorStyle;
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
  screen: TerminalPtr,
  tree: TreePtr,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let size = U16Size::new(cols, rows);
    let screen = Terminal::new(size, Default::default());
    let screen = make_terminal_ptr(screen);
    let tree = Tree::new(Arc::downgrade(&screen));
    let tree = make_tree_ptr(tree);

    let root_widget = RootWidget::new(size);
    let root_widget_node = make_node_ptr(Node::RootWidgetNode(root_widget));
    tree.read().unwrap().insert_root_node(
      root_widget_node.read().unwrap().id(),
      root_widget_node.clone(),
    );
    let window = Window::new(
      IRect::new((0, 0), (size.width() as isize, size.height() as isize)),
      0,
    );
    let window_node = make_node_ptr(Node::WindowNode(window));
    tree.read().unwrap().insert_node(
      window_node.read().unwrap().id(),
      window_node.clone(),
      root_widget_node.read().unwrap().id(),
    );

    let cursor = Cursor::new(
      point!(x:0,y:0),
      true,
      false,
      SetCursorStyle::DefaultUserShape,
    );
    let cursor_node = make_node_ptr(Node::CursorNode(cursor));
    tree.read().unwrap().insert_node(
      cursor_node.read().unwrap().id(),
      cursor_node.clone(),
      window_node.read().unwrap().id(),
    );

    Ok(EventLoop { screen, tree })
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
