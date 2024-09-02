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
use std::io::{BufWriter, Stdout};
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
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{Cursor, RootContainer, Widgetable, Window};

#[derive(Debug)]
pub struct EventLoop {
  pub cli_opt: CliOpt,
  pub canvas: CanvasArc,
  pub tree: TreeArc,
  pub state: StateArc,
  pub buffers: BuffersArc,
  pub writer: BufWriter<Stdout>,
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
      writer: BufWriter::new(std::io::stdout()),
    })
  }

  pub async fn init(&mut self) -> IoResult<()> {
    self.queue_cursor().await?;
    self.writer.flush()?;

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
        .handle(self.tree.clone(), self.buffers.clone(), event)
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

    self.queue_shader(shader).await?;

    self.writer.flush()?;

    Ok(true)
  }

  /// Put (render) canvas shader.
  async fn queue_shader(&mut self, shader: Shader) -> IoResult<()> {
    for shader_command in shader.iter() {
      match shader_command {
        ShaderCommand::CursorSetCursorStyle(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorDisableBlinking(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorEnableBlinking(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorHide(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveDown(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveLeft(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveRight(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveTo(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveToColumn(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveToNextLine(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveToPreviousLine(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveToRow(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorMoveUp(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorRestorePosition(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorSavePosition(command) => queue!(self.writer, command)?,
        ShaderCommand::CursorShow(command) => queue!(self.writer, command)?,
        ShaderCommand::EventDisableBracketedPaste(command) => queue!(self.writer, command)?,
        ShaderCommand::EventDisableFocusChange(command) => queue!(self.writer, command)?,
        ShaderCommand::EventDisableMouseCapture(command) => queue!(self.writer, command)?,
        ShaderCommand::EventEnableBracketedPaste(command) => queue!(self.writer, command)?,
        ShaderCommand::EventEnableFocusChange(command) => queue!(self.writer, command)?,
        ShaderCommand::EventEnableMouseCapture(command) => queue!(self.writer, command)?,
        ShaderCommand::EventPopKeyboardEnhancementFlags(command) => queue!(self.writer, command)?,
        ShaderCommand::EventPushKeyboardEnhancementFlags(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleResetColor(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetAttribute(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetAttributes(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetBackgroundColor(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetColors(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetForegroundColor(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetStyle(command) => queue!(self.writer, command)?,
        ShaderCommand::StyleSetUnderlineColor(command) => queue!(self.writer, command)?,
        ShaderCommand::StylePrintStyledContentString(command) => queue!(self.writer, command)?,
        ShaderCommand::StylePrintString(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalBeginSynchronizedUpdate(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalClear(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalDisableLineWrap(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalEnableLineWrap(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalEndSynchronizedUpdate(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalEnterAlternateScreen(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalLeaveAlternateScreen(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalScrollDown(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalScrollUp(command) => queue!(self.writer, command)?,
        ShaderCommand::TerminalSetSize(command) => queue!(self.writer, command)?,
      }
    }

    Ok(())
  }

  /// Put (render) canvas cursor.
  async fn queue_cursor(&mut self) -> IoResult<()> {
    let cursor = *self
      .canvas
      .try_read_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
      .unwrap()
      .frame()
      .cursor();

    if cursor.blinking() {
      queue!(self.writer, crossterm::cursor::EnableBlinking)?;
    } else {
      queue!(self.writer, crossterm::cursor::DisableBlinking)?;
    }
    if cursor.hidden() {
      queue!(self.writer, crossterm::cursor::Hide)?;
    } else {
      queue!(self.writer, crossterm::cursor::Show)?;
    }

    queue!(self.writer, cursor.style())?;
    queue!(
      self.writer,
      crossterm::cursor::MoveTo(cursor.pos().x(), cursor.pos().y())
    )?;

    Ok(())
  }
}
