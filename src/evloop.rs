//! Main event loop for TUI application.

#![allow(unused_imports, dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode, KeyEventKind, KeyEventState, KeyModifiers,
};
use crossterm::{self, queue, terminal};
use futures::StreamExt;
use geo::point;
use std::collections::HashMap;
// use heed::types::U16;
use futures::stream::FuturesUnordered;
use parking_lot::ReentrantMutexGuard;
use parking_lot::RwLock;
use ropey::RopeBuilder;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::Write;
use std::io::{BufWriter, Stdout};
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::{AbortHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error};

use crate::buf::{Buffer, Buffers, BuffersArc};
use crate::cart::{IRect, Size, U16Rect, U16Size, URect};
use crate::cli::CliOpt;
use crate::evloop::msg::{Dummy, WorkerToMasterMessage};
use crate::evloop::task::{TaskResult, TaskableDataAccess};
use crate::geo_size_as;
use crate::glovar;
use crate::result::{IoResult, VoidIoResult};
use crate::rt::msg::{EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::rt::{JsDataAccess, JsRuntime};
use crate::state::fsm::{QuitStateful, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc, CursorStyle, Shader, ShaderCommand};
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{Cursor, RootContainer, Widgetable, Window};

pub mod msg;
pub mod task;

#[derive(Debug)]
pub struct EventLoop {
  pub cli_opt: CliOpt,

  // UI
  pub canvas: CanvasArc,
  pub tree: TreeArc,
  pub writer: BufWriter<Stdout>,

  // State
  pub state: StateArc,

  // Buffers
  pub buffers: BuffersArc,

  // Spawned tasks inside running loop.
  // Here name the spawned tasks "worker", the main loop thread "master".
  pub cancellation_token: CancellationToken,
  pub task_tracker: TaskTracker,
  // Channels that workers send messages to master.
  pub worker_send_to_master: UnboundedSender<WorkerToMasterMessage>,
  pub master_recv_from_worker: UnboundedReceiver<WorkerToMasterMessage>,

  // Channels between this running loop and js runtime.
  evloop_send_to_js: UnboundedSender<EventLoopToJsRuntimeMessage>,
  evloop_recv_from_js: UnboundedReceiver<JsRuntimeToEventLoopMessage>,
}

impl EventLoop {
  pub fn new(
    cli_opt: CliOpt,
    evloop_send_to_js: UnboundedSender<EventLoopToJsRuntimeMessage>,
    evloop_recv_from_js: UnboundedReceiver<JsRuntimeToEventLoopMessage>,
  ) -> IoResult<Self> {
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

    // Sender/receiver
    let (worker_send_to_master, master_recv_from_worker) = unbounded_channel();

    Ok(EventLoop {
      cli_opt,
      canvas,
      tree: Tree::to_arc(tree),
      state: State::to_arc(state),
      buffers: Buffers::to_arc(buffers),
      writer: BufWriter::new(std::io::stdout()),
      cancellation_token: CancellationToken::new(),
      task_tracker: TaskTracker::new(),
      worker_send_to_master,
      master_recv_from_worker,
      evloop_send_to_js,
      evloop_recv_from_js,
    })
  }

  pub fn init(&mut self) -> VoidIoResult {
    self.queue_cursor()?;
    self.writer.flush()?;

    // Has input files.
    if !self.cli_opt.file().is_empty() {
      let data_access = TaskableDataAccess::new(
        self.state.clone(),
        self.tree.clone(),
        self.buffers.clone(),
        self.worker_send_to_master.clone(),
      );
      let input_files = self.cli_opt.file().to_vec();
      self.task_tracker.spawn(async move {
        let (default_input_file, other_input_files) = input_files.split_first().unwrap();
        let default_input_file = default_input_file.clone();
        if task::startup::input_files::edit_default_file(data_access.clone(), default_input_file)
          .await
          .is_ok()
          && !other_input_files.is_empty()
        {
          let other_input_files = other_input_files.to_vec();
          let _ =
            task::startup::input_files::edit_other_files(data_access.clone(), other_input_files)
              .await;
        }
      });
    }

    Ok(())
  }

  async fn process_event(&mut self, next_event: Option<IoResult<Event>>) -> bool {
    match next_event {
      Some(Ok(event)) => {
        debug!("Polled_terminal event ok: {:?}", event);

        // Handle by state machine
        let state_response = {
          self
            .state
            .try_write_for(Duration::from_secs(glovar::MUTEX_TIMEOUT()))
            .unwrap()
            .handle(self.tree.clone(), self.buffers.clone(), event)
        };

        // Exit loop and quit.
        if let StatefulValue::QuitState(_) = state_response.next_stateful {
          self.cancellation_token.cancel();
          return false;
        }

        return true;
      }
      Some(Err(e)) => {
        error!("Polled terminal event error: {:?}", e);
      }
      None => {
        error!("Terminal event stream is exhausted, exit loop");
      }
    }
    false
  }

  async fn process_notify(
    &mut self,
    received_notifications: &mut Vec<WorkerToMasterMessage>,
    received_count: usize,
  ) -> bool {
    debug!("Received {:?} notifications from workers", received_count);
    received_notifications.clear();
    true
  }

  pub async fn run(&mut self) -> VoidIoResult {
    let mut reader = EventStream::new();
    let received_limit = 100_usize;
    let mut received_notifies: Vec<WorkerToMasterMessage> = Vec::with_capacity(received_limit);
    loop {
      tokio::select! {
        // Receive keyboard/mouse events
        next_event = reader.next() => {
            if !self.process_event(next_event).await {
                break;
            }
        }
        // Receive notification from workers
        received_count = self.master_recv_from_worker.recv_many(&mut received_notifies, received_limit) => {
            if !self.process_notify(&mut received_notifies, received_count).await {
                break;
            }
        }
        // Receive cancellation notify
        _ = self.cancellation_token.cancelled() => {
            debug!("Receive cancellation token, exit loop");
            // break;
        }
      }

      // Update terminal
      self.render()?;
    }
    Ok(())
  }

  fn render(&mut self) -> VoidIoResult {
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

    self.queue_shader(shader)?;

    self.writer.flush()?;

    Ok(())
  }

  /// Put (render) canvas shader.
  fn queue_shader(&mut self, shader: Shader) -> VoidIoResult {
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
  fn queue_cursor(&mut self) -> VoidIoResult {
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
