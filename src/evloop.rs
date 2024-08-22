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

use crate::buf::{Buffer, Buffers, BuffersArc};
use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::cli::CliOpt;
use crate::geo_size_as;
use crate::glovar;
use crate::state::fsm::{QuitStateful, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc, CursorStyle, Shader, ShaderCommand};
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{Cursor, RootContainer, Widget, WidgetId, Window};

#[derive(Clone)]
pub struct EventLoop {
  pub cli_opt: CliOpt,
  pub canvas: CanvasArc,
  pub tree: TreeArc,
  pub state: StateArc,
  pub buffers: BuffersArc,
}

impl EventLoop {
  pub async fn new(cli_opt: CliOpt) -> IoResult<Self> {
    // Canvas
    let (cols, rows) = terminal::size()?;
    let canvas_size = U16Size::new(cols, rows);
    let canvas = Canvas::new(canvas_size);
    let canvas = Canvas::to_arc(canvas);

    // UI Tree
    let mut tree = Tree::new(canvas_size);
    debug!("new, screen size: {:?}", canvas_size);

    // Buffers
    let mut buffers = Buffers::new();
    let buffer = Buffer::to_arc(Buffer::new());
    buffers.insert(buffer.clone());

    let window_shape = IRect::new(
      (0, 0),
      (canvas_size.width() as isize, canvas_size.height() as isize),
    );
    let window = Window::new(window_shape, Arc::downgrade(&buffer));
    let window_id = window.id();
    let window_node = TreeNode::Window(window);
    tree.bounded_insert(&tree.root_id(), window_node);
    debug!("new, insert window container: {:?}", window_id);

    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor = Cursor::new(cursor_shape);
    let cursor_id = cursor.id();
    let cursor_node = TreeNode::Cursor(cursor);
    tree.bounded_insert(&window_id, cursor_node);

    // State
    let state = State::default();

    Ok(EventLoop {
      cli_opt,
      canvas,
      tree: Tree::to_arc(tree),
      state: State::to_arc(state),
      buffers: Buffers::to_arc(buffers),
    })
  }

  pub async fn init(&mut self) -> IoResult<()> {
    let mut out = std::io::stdout();

    let cursor = self
      .canvas
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
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
        let buffers = raw_self.as_ref().buffers.clone();
        static RBUF_SIZE: usize = if std::mem::size_of::<usize>() < 8 {
          4096
        } else {
          8192
        };

        tokio::spawn(async move {
          let mut _current_window_updated = false;

          for (i, one_file) in cli_opt.file().iter().enumerate() {
            debug!("Read the {} input file: {:?}", i, one_file);
            match fs::File::open(one_file).await {
              Ok(mut file) => {
                let mut builder = RopeBuilder::new();

                let mut rbuf: Vec<u8> = vec![0_u8; RBUF_SIZE];
                debug!("Read buffer bytes size: {}", rbuf.len());
                loop {
                  match file.read_buf(&mut rbuf).await {
                    Ok(n) => {
                      debug!("Read {} bytes", n);
                      let rbuf1: &[u8] = &rbuf;
                      let rbuf_str: String = String::from_utf8_lossy(rbuf1).into_owned();

                      builder.append(&rbuf_str.to_owned());
                      if n == 0 {
                        // Finish reading, create new buffer
                        let buffer = Buffer::from(builder);
                        buffers
                          .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
                          .unwrap()
                          .insert(Buffer::to_arc(buffer));

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
        .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .handle(self.tree.clone(), event)
    };

    // Exit loop and quit.
    if let StatefulValue::QuitState(_) = state_response.next_stateful {
      return Ok(false);
    }

    {
      // Draw UI components to the canvas.
      self
        .tree
        .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
        .unwrap()
        .draw(self.canvas.clone());
    }

    let shader = {
      // Compute the commands that need to output to the terminal device.
      self
        .canvas
        .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
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
