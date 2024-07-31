//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]

use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::geo_size_as;
use crate::ui::frame::CursorStyle;
use crate::ui::term::{make_terminal_ptr, Terminal, TerminalPtr};
use crate::ui::tree::{Tree, TreeArc, TreeNode, TreeNodeArc};
use crate::ui::widget::{
  Cursor, RootContainer, Widget, WidgetEnum, WindowContainer, WindowContent,
};
use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode, KeyEventKind, KeyEventState, KeyModifiers,
};
use crossterm::{cursor as termcursor, queue, terminal};
use futures::StreamExt;
use geo::point;
use heed::types::U16;
use std::borrow::Borrow;
use std::io::{Result as IoResult, Write};
use std::sync::{Arc, RwLock};
use tracing::{debug, error};

pub struct EventLoop {
  screen: TerminalPtr,
  tree: TreeArc,
}

impl EventLoop {
  pub async fn new() -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let screen_size = U16Size::new(cols, rows);
    let screen = Terminal::new(screen_size);
    let screen = make_terminal_ptr(screen);
    let mut tree = Tree::new(Arc::downgrade(&screen));

    let root_container = RootContainer::default();
    let root_container_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    let root_container_node = TreeNode::new(
      None,
      WidgetEnum::RootContainer(root_container),
      root_container_shape,
    );
    let root_container_node = TreeNode::arc(root_container_node);
    tree.insert(None, root_container_node.clone());

    let window_container = WindowContainer::default();
    let window_container_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    let window_container_node = TreeNode::new(
      Some(Arc::downgrade(&root_container_node)),
      WidgetEnum::WindowContainer(window_container),
      window_container_shape,
    );
    let window_container_node = TreeNode::arc(window_container_node);
    tree.insert(
      Some(root_container_node.clone()),
      window_container_node.clone(),
    );

    let window_content = WindowContent::default();
    let window_content_shape = IRect::new(
      (0, 0),
      (screen_size.width() as isize, screen_size.height() as isize),
    );
    let window_content_node = TreeNode::new(
      Some(Arc::downgrade(&window_container_node)),
      WidgetEnum::WindowContent(window_content),
      window_content_shape,
    );
    let window_content_node = TreeNode::arc(window_content_node);
    tree.insert(
      Some(window_container_node.clone()),
      window_content_node.clone(),
    );

    let cursor = Cursor::default();
    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor_node = TreeNode::new(
      Some(Arc::downgrade(&window_content_node)),
      WidgetEnum::Cursor(cursor),
      cursor_shape,
    );
    let cursor_node = TreeNode::arc(cursor_node);
    tree.insert(Some(window_container_node.clone()), cursor_node.clone());

    Ok(EventLoop {
      screen,
      tree: Tree::arc(tree),
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
    queue!(out, termcursor::MoveTo(cursor.pos.x(), cursor.pos.y()))?;

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
            error!("Error: {:?}\r", e);
            break;
          },
          None => break,
        }
      }
    }
    Ok(())
  }

  pub async fn accept(&mut self, event: Event) -> bool {
    debug!("Event::{:?}", event);

    match event {
      Event::FocusGained => {}
      Event::FocusLost => {}
      Event::Key(key_event) => match key_event.kind {
        KeyEventKind::Press => {}
        KeyEventKind::Repeat => {}
        KeyEventKind::Release => {}
      },
      Event::Mouse(_mouse_event) => {}
      Event::Paste(_paste_string) => {}
      Event::Resize(_columns, _rows) => {}
    }

    // if event == Event::Key(KeyCode::Char('c').into()) {
    //   println!("Curosr position: {:?}\r", termcursor::position());
    // }

    // quit loop
    if event == Event::Key(KeyCode::Esc.into()) {
      return false;
    }

    // continue loop
    true
  }
}
