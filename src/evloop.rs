//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode, KeyEventKind, KeyEventState, KeyModifiers,
};
use crossterm::{self, queue, terminal};
use futures::StreamExt;
use geo::point;
// use heed::types::U16;
use parking_lot::ReentrantMutexGuard;
use ropey::RopeBuilder;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::{Result as IoResult, Write};
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::buffer::Buffer;
use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::cli::CliOpt;
use crate::geo_size_as;
use crate::glovar;
use crate::state::fsm::{QuitStateful, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::frame::CursorStyle;
use crate::ui::term::{Shader, ShaderCommand, Terminal, TerminalArc};
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{
  Cursor, RootContainer, Widget, WidgetValue, WindowContainer, WindowContent,
};

#[derive(Clone)]
pub struct EventLoop {
  pub cli_opt: CliOpt,
  pub screen: TerminalArc,
  pub tree: TreeArc,
  pub state: StateArc,
}

impl EventLoop {
  pub async fn new(cli_opt: CliOpt) -> IoResult<Self> {
    let (cols, rows) = terminal::size()?;
    let screen_size = U16Size::new(cols, rows);
    let screen = Terminal::new(screen_size);
    let screen = Terminal::to_arc(screen);
    let mut tree = Tree::new(screen_size);
    let mut state = State::new();
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
    tree.insert(&tree.root_id(), window_container_node);
    state.set_current_window_widget(Some(window_container_id));
    state.window_widgets_mut().insert(window_container_id);
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
    tree.insert(&window_container_id, window_content_node);
    debug!("new, insert window content: {:?}", window_content_id);

    let cursor = Cursor::new();
    let cursor_id = cursor.id();
    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor_node = TreeNode::new(WidgetValue::Cursor(cursor), cursor_shape);
    tree.insert(&window_content_id, cursor_node);
    state.set_cursor_widget(Some(cursor_id));

    Ok(EventLoop {
      cli_opt,
      screen,
      tree: Tree::to_arc(tree),
      state: State::to_arc(state),
    })
  }

  pub async fn init(&mut self) -> IoResult<()> {
    let mut out = std::io::stdout();

    let cursor = self
      .screen
      .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .frame()
      .cursor;

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

    // Has input files.
    if !self.cli_opt.file().is_empty() {
      unsafe {
        // Fix `self` lifetime requires 'static in spawn.
        let raw_self = NonNull::new(self as *mut EventLoop).unwrap();
        let cli_opt = raw_self.as_ref().cli_opt.clone();
        let state = raw_self.as_ref().state.clone();

        tokio::spawn(async move {
          for (i, one_file) in cli_opt.file().iter().enumerate() {
            debug!("Read the {} input file: {:?}", i, one_file);
            match fs::File::open(one_file).await {
              Ok(mut file) => {
                let mut builder = RopeBuilder::new();

                let mut rbuf: Vec<u8> = vec![0_u8; std::mem::size_of::<usize>()];
                loop {
                  match file.read_buf(&mut rbuf).await {
                    Ok(n) => {
                      debug!("Read {} bytes", n);
                      let rbuf1: &[u8] = &rbuf;
                      let rbuf_str = String::from_utf8_lossy(rbuf1).into_owned();
                      builder.append(&rbuf_str.to_owned());
                      if n == 0 {
                        // Finish reading
                        let buffer = Buffer::from(builder);
                        state
                          .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                          .unwrap()
                          .buffers_mut()
                          .insert(buffer.id, buffer);
                        // println!("Read file {:?} into buffer", input_file);
                        break;
                      }
                    }
                    Err(e) => {
                      // Unexpected error
                      println!("Failed to read file {:?} with error {:?}", one_file, e);
                    }
                  }
                }
              }
              Err(e) => {
                println!("Failed to open file {:?} with error {:?}", one_file, e);
              }
            }
          }
        });
      }
    }

    Ok(())
  }

  pub async fn run(&mut self) -> IoResult<()> {
    let mut reader = EventStream::new();
    loop {
      tokio::select! {
        polled_event = reader.next() => match polled_event {
          Some(Ok(event)) => {
            debug!("polled_event ok: {:?}", event);
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
            debug!("polled_event error: {:?}", e);
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
    debug!("event: {:?}", event);
    // println!("Event:{:?}", event);

    let state_response = {
      self
        .state
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .handle(self.tree.clone(), event)
    };

    match state_response.next_stateful {
      StatefulValue::QuitState(_) => {
        return Ok(false);
      }
      _ => { /*Skip*/ }
    }

    {
      // Draw UI components to the terminal frame.
      self
        .tree
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .draw(self.screen.clone());
    }

    let shader = {
      // Compute the commands that need to output to the terminal device.
      self
        .screen
        .try_lock_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .shade()
    };

    self.render(shader).await?;

    Ok(true)
  }

  async fn render(&mut self, shader: Shader) -> IoResult<()> {
    let mut out = std::io::stdout();
    for shader_command in shader.iter() {
      match shader_command {
        ShaderCommand::CursorSetCursorStyle(command) => queue!(out, command)?,
        ShaderCommand::CursorDisableBlinking(command) => queue!(out, command)?,
        ShaderCommand::CursorEnableBlinking(command) => queue!(out, command)?,
        ShaderCommand::CursorHide(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveDown(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveLeft(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveRight(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveTo(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveToColumn(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveToNextLine(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveToPreviousLine(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveToRow(command) => queue!(out, command)?,
        ShaderCommand::CursorMoveUp(command) => queue!(out, command)?,
        ShaderCommand::CursorRestorePosition(command) => queue!(out, command)?,
        ShaderCommand::CursorSavePosition(command) => queue!(out, command)?,
        ShaderCommand::CursorShow(command) => queue!(out, command)?,
        ShaderCommand::EventDisableBracketedPaste(command) => queue!(out, command)?,
        ShaderCommand::EventDisableFocusChange(command) => queue!(out, command)?,
        ShaderCommand::EventDisableMouseCapture(command) => queue!(out, command)?,
        ShaderCommand::EventEnableBracketedPaste(command) => queue!(out, command)?,
        ShaderCommand::EventEnableFocusChange(command) => queue!(out, command)?,
        ShaderCommand::EventEnableMouseCapture(command) => queue!(out, command)?,
        ShaderCommand::EventPopKeyboardEnhancementFlags(command) => queue!(out, command)?,
        ShaderCommand::EventPushKeyboardEnhancementFlags(command) => queue!(out, command)?,
        ShaderCommand::StyleResetColor(command) => queue!(out, command)?,
        ShaderCommand::StyleSetAttribute(command) => queue!(out, command)?,
        ShaderCommand::StyleSetAttributes(command) => queue!(out, command)?,
        ShaderCommand::StyleSetBackgroundColor(command) => queue!(out, command)?,
        ShaderCommand::StyleSetColors(command) => queue!(out, command)?,
        ShaderCommand::StyleSetForegroundColor(command) => queue!(out, command)?,
        ShaderCommand::StyleSetStyle(command) => queue!(out, command)?,
        ShaderCommand::StyleSetUnderlineColor(command) => queue!(out, command)?,
        ShaderCommand::TerminalBeginSynchronizedUpdate(command) => queue!(out, command)?,
        ShaderCommand::TerminalClear(command) => queue!(out, command)?,
        ShaderCommand::TerminalDisableLineWrap(command) => queue!(out, command)?,
        ShaderCommand::TerminalEnableLineWrap(command) => queue!(out, command)?,
        ShaderCommand::TerminalEndSynchronizedUpdate(command) => queue!(out, command)?,
        ShaderCommand::TerminalEnterAlternateScreen(command) => queue!(out, command)?,
        ShaderCommand::TerminalLeaveAlternateScreen(command) => queue!(out, command)?,
        ShaderCommand::TerminalScrollDown(command) => queue!(out, command)?,
        ShaderCommand::TerminalScrollUp(command) => queue!(out, command)?,
        ShaderCommand::TerminalSetSize(command) => queue!(out, command)?,
      }
    }

    out.flush()?;

    Ok(())
  }
}
