//! Event loop.

use crate::buf::{BuffersManager, BuffersManagerArc};
use crate::cli::CliOptions;
use crate::command::{ExCommandsManager, ExCommandsManagerArc};
use crate::content::{TextContents, TextContentsArc};
use crate::js::{self, JsRuntime, JsRuntimeOptions, SnapshotData};
use crate::msg::{self, JsMessage, MasterMessage};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::cmdline_ops;
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::*;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use compact_str::ToCompactString;
use writer::{StdoutWritable, StdoutWriterValue};

use crate::ui::widget::command_line::CommandLine;
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[cfg(test)]
use crate::tests::evloop::MockReader;
#[cfg(test)]
use bitflags::bitflags_match;
#[cfg(test)]
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

pub mod writer;

#[derive(Debug)]
/// For slow tasks that are suitable to put in the background, this event loop
/// will spawn them in tokio's async tasks and let them sync back data once
/// they are done. The event loop controls all the tasks with
/// [`CancellationToken`] and [`TaskTracker`].
///
/// Js runtime and event loop also communicate via channels, since js runtime
/// is based on v8 engine, all v8 APIs are not thread-safe.
pub struct EventLoop {
  /// Indicates the start time of the process.
  pub startup_moment: Instant,
  /// Specifies the timestamp which the current process began in Unix time.
  pub startup_unix_epoch: u128,

  /// Command line options.
  pub cli_opts: CliOptions,

  /// Stdout writer for editor mode TUI.
  pub writer: StdoutWriterValue,

  /// Widget tree for UI.
  pub tree: TreeArc,
  /// Canvas for UI.
  pub canvas: CanvasArc,

  /// (Global) editing state.
  pub state: StateArc,
  /// Finite-state machine for editing state.
  pub stateful_machine: StatefulValue,

  /// Vim buffers.
  pub buffers: BuffersManagerArc,
  /// Text contents (except buffers).
  pub contents: TextContentsArc,
  /// Ex commands.
  pub commands: ExCommandsManagerArc,

  /// Cancellation token to notify the main loop to exit.
  pub cancellation_token: CancellationToken,
  /// Task tracker for spawned tasks, there are two trackers:
  ///
  /// 1. Cancellable/deteched tracker for those tasks that are safe to cancel.
  /// 2. Block tracker are for dangerous tasks, user will have to wait for them complete before
  ///    exit the editor.
  ///
  /// Most write file operations are spawned with block tracker to ensure they will be safely
  /// complete to avoid damage user data files. While for most reading operations and pure CPU
  /// calculations, they will be cancelled when editor exit.
  pub detached_tracker: TaskTracker,
  pub blocked_tracker: TaskTracker,

  /// Js runtime.
  pub js_runtime: JsRuntime,

  /// Channel-1
  pub master_tx: Sender<MasterMessage>,
  pub master_rx: Receiver<MasterMessage>,

  /// Channel-2
  pub jstick_tx: Sender<JsMessage>,
  pub jstick_rx: Receiver<JsMessage>,

  /// Channel-3
  pub jsrt_tx: Sender<JsMessage>,
  // pub jsrt_rx: Receiver<JsMessage>,
}

#[cfg(test)]
fn is_ctrl_d(event: &Option<IoResult<Event>>) -> bool {
  match event {
    Some(Ok(Event::Key(key_event))) => {
      if key_event.code == KeyCode::Char('d')
        && key_event.kind == KeyEventKind::Press
      {
        bitflags_match!(key_event.modifiers, {
          KeyModifiers::CONTROL => true,
          _ => false
        })
      } else {
        false
      }
    }
    _ => false,
  }
}

impl EventLoop {
  #[allow(clippy::type_complexity)]
  pub fn _internal_new(
    terminal_cols: u16,
    terminal_rows: u16,
  ) -> IoResult<(
    /* startup_moment */ Instant,
    /* startup_unix_epoch */ u128,
    /* canvas */ CanvasArc,
    /* tree */ TreeArc,
    /* state */ StateArc,
    /* stateful_machine */ StatefulValue,
    /* buffers */ BuffersManagerArc,
    /* contents */ TextContentsArc,
    /* commands */ ExCommandsManagerArc,
    /* cancellation_token */ CancellationToken,
    /* detached_tracker */ TaskTracker,
    /* blocked_tracker */ TaskTracker,
    (
      /* master_tx */ Sender<MasterMessage>,
      /* master_rx */ Receiver<MasterMessage>,
    ),
    (
      /* jstick_tx */ Sender<JsMessage>,
      /* jstick_rx */ Receiver<JsMessage>,
    ),
    (
      /* jsrt_tx */ Sender<JsMessage>,
      /* jsrt_rx */ Receiver<JsMessage>,
    ),
  )> {
    // Canvas
    let canvas_size = U16Size::new(terminal_cols, terminal_rows);
    let canvas = Canvas::new(canvas_size);
    let canvas = Canvas::to_arc(canvas);

    // UI Tree
    let tree = Tree::to_arc(Tree::new(canvas_size));

    // Buffers
    let buffers_manager = BuffersManager::to_arc(BuffersManager::new());
    let text_contents = TextContents::to_arc(TextContents::new(canvas_size));
    let ex_commands_manager =
      ExCommandsManager::to_arc(ExCommandsManager::new());

    // State
    let state = State::to_arc(State::new());
    let stateful_machine = StatefulValue::default();

    // When implements `Promise`, `async`/`await` APIs for javascript runtime,
    // we need to leverage tokio's async runtime. i.e. first we send js task
    // requests to master (i.e. "this" event loop, here call it "master"), let
    // the master handles these tasks with tokio's async tasks. Once a task is
    // done, we send js task results back to js runtime.
    //
    // These tasks are low-level infrastructures, for example:
    //
    // - File IO
    // - Network
    // - Timer
    // - And more...
    //
    // But there are technical limitations that we cannot use tokio APIs along
    // with V8 engine, since V8 rust bindings are not Arc/Mutex (i.e. not
    // thread safe), while tokio async runtime requires Arc/Mutex (i.e. thread
    // safe).
    //
    // The request/response data flow uses below message channels:
    //
    // - Channel-1 `master_tx` => `master_rx` on `MasterMessage`.
    // - Channel-2 `jstick_tx` => `jstick_rx` on `JsMessage`.
    // - Channel-3 `jsrt_tx` => `jsrt_rx` on `JsMessage`.
    //
    // The dataflow follows below steps:
    //
    // 1. Js runtime --- [`MasterMessage`] (channel-1) --> Event loop
    // 2. Event loop handles js requests with tokio async tasks
    // 3. Event loop --- JsMessage (channel-2) --> Event loop
    // 4. Event loop --- JsMessage (channel-3) --> Js runtime
    // 5. Js runtime completes all async results.
    //
    // NOTE: You must notice, the step-3 and channel-2 seems unnecessary. Yes,
    // they're simply for trigger the `tokio::select!` loop.

    // Channel-1: js runtime => master
    let (master_tx, master_rx) = channel(*CHANNEL_BUF_SIZE);
    // Channel-2
    let (jstick_tx, jstick_rx) = channel(*CHANNEL_BUF_SIZE);
    // Channel-3
    let (jsrt_tx, jsrt_rx) = channel(*CHANNEL_BUF_SIZE);

    // Startup time
    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    Ok((
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers_manager,
      text_contents,
      ex_commands_manager,
      CancellationToken::new(),
      TaskTracker::new(),
      TaskTracker::new(),
      (master_tx, master_rx),
      (jstick_tx, jstick_rx),
      (jsrt_tx, jsrt_rx),
    ))
  }

  /// Make new event loop.
  pub fn new(cli_opts: CliOptions, snapshot: SnapshotData) -> IoResult<Self> {
    let (cols, rows) = crossterm::terminal::size()?;
    let (
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      (master_tx, master_rx),
      (jstick_tx, jstick_rx),
      (jsrt_tx, jsrt_rx),
    ) = Self::_internal_new(cols, rows)?;

    let writer = if cli_opts.headless() {
      StdoutWriterValue::headless()
    } else {
      StdoutWriterValue::editor()
    };

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      snapshot,
      startup_moment,
      startup_unix_epoch,
      master_tx.clone(),
      jsrt_rx,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
      commands.clone(),
      state.clone(),
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers,
      contents,
      commands,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      js_runtime,
      master_tx,
      master_rx,
      jstick_tx,
      jstick_rx,
      jsrt_tx,
    })
  }

  #[cfg(test)]
  /// Make new event loop for testing.
  pub fn mock_new(
    terminal_columns: u16,
    terminal_rows: u16,
    cli_opts: CliOptions,
  ) -> IoResult<Self> {
    let (
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      (master_tx, master_rx),
      (jstick_tx, jstick_rx),
      (jsrt_tx, jsrt_rx),
    ) = Self::_internal_new(terminal_columns, terminal_rows)?;

    let writer = StdoutWriterValue::dev_null();

    // Js Runtime
    let js_runtime = JsRuntime::new_without_snapshot(
      JsRuntimeOptions::default(),
      startup_moment,
      startup_unix_epoch,
      master_tx.clone(),
      jsrt_rx,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
      commands.clone(),
      state.clone(),
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers,
      contents,
      commands,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      js_runtime,
      master_tx,
      master_rx,
      jstick_tx,
      jstick_rx,
      jsrt_tx,
    })
  }

  /// Initialize the editor
  pub fn initialize(&mut self) -> IoResult<()> {
    self._init_config()?;

    self.writer.init()?;

    self._init_buffers()?;
    self._init_windows()?;

    // Flush logic UI to terminal, i.e. print UI to stdout
    lock!(self.tree).draw(self.canvas.clone());
    self.writer.init_complete(&mut lock!(self.canvas))?;

    Ok(())
  }

  /// Initialize user config file.
  fn _init_config(&mut self) -> IoResult<()> {
    if let Some(config_entry) = PATH_CONFIG.config_entry() {
      match self
        .js_runtime
        .execute_module(config_entry.to_str().unwrap(), None)
      {
        Ok(_) => { /* do nothing */ }
        Err(e) => {
          // Send error message to command-line
          let current_handle = tokio::runtime::Handle::current();
          let master_tx = self.master_tx.clone();
          let message_id = js::next_future_id();
          let e = e.to_compact_string();
          current_handle.spawn_blocking(move || {
            master_tx
              .blocking_send(MasterMessage::PrintReq(msg::PrintReq::new(
                message_id, e,
              )))
              .unwrap();
          });
        }
      }
    }
    Ok(())
  }

  /// Initialize buffers.
  pub fn _init_buffers(&mut self) -> IoResult<()> {
    let canvas_size = lock!(self.canvas).size();

    // Create default buffer from `FILES` arguments from cli, or with an empty buffer.
    let input_files = &self.cli_opts.file();
    if !input_files.is_empty() {
      for input_file in input_files.iter() {
        let maybe_buf_id =
          lock!(self.buffers).new_file_buffer(canvas_size, input_file);
        match maybe_buf_id {
          Ok(buf_id) => {
            trace!("Created file buffer {:?}:{:?}", input_file, buf_id);
          }
          Err(e) => {
            error!("Failed to create file buffer {:?}:{:?}", input_file, e);
          }
        }
      }
    } else {
      let buf_id = lock!(self.buffers).new_empty_buffer(canvas_size);
      trace!("Created empty buffer {:?}", buf_id);
    }

    Ok(())
  }

  /// Initialize windows.
  pub fn _init_windows(&mut self) -> IoResult<()> {
    // Initialize default window, with default buffer.
    let (canvas_size, canvas_cursor) = {
      let canvas = lock!(self.canvas);
      let canvas_size = canvas.size();
      let canvas_cursor = *canvas.frame().cursor();
      (canvas_size, canvas_cursor)
    };
    let mut tree = lock!(self.tree);
    let tree_root_id = tree.root_id();
    let window_shape = IRect::new(
      (0, 0),
      (
        canvas_size.width() as isize,
        canvas_size.height().saturating_sub(1) as isize,
      ),
    );
    let mut window = {
      let buffers = lock!(self.buffers);
      let (buf_id, buf) = buffers.first_key_value().unwrap();
      trace!("Bind first buffer to default window {:?}", buf_id);
      Window::new(
        tree.global_local_options(),
        window_shape,
        Arc::downgrade(buf),
      )
    };
    let window_id = window.id();

    // Initialize cursor inside the default window.
    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor = Cursor::new(
      cursor_shape,
      canvas_cursor.blinking(),
      canvas_cursor.hidden(),
      canvas_cursor.style(),
    );
    let _previous_inserted_cursor = window.insert_cursor(cursor);
    debug_assert!(_previous_inserted_cursor.is_none());

    tree.bounded_insert(tree_root_id, TreeNode::Window(window));
    tree.set_current_window_id(Some(window_id));

    // Initialize default command-line.
    let cmdline_shape = IRect::new(
      (0, canvas_size.height().saturating_sub(1) as isize),
      (canvas_size.width() as isize, canvas_size.height() as isize),
    );
    let cmdline =
      CommandLine::new(cmdline_shape, Arc::downgrade(&self.contents));

    tree.bounded_insert(tree_root_id, TreeNode::CommandLine(cmdline));

    Ok(())
  }

  /// Shutdown.
  pub fn shutdown(&self) -> IoResult<()> {
    self.writer.shutdown()?;

    Ok(())
  }

  async fn process_event(&mut self, event: Option<IoResult<Event>>) {
    match event {
      Some(Ok(event)) => {
        trace!("Polled terminal event ok: {:?}", event);

        let data_access = StatefulDataAccess::new(
          event,
          self.state.clone(),
          self.tree.clone(),
          self.buffers.clone(),
          self.contents.clone(),
          self.commands.clone(),
          self.master_tx.clone(),
          self.jstick_tx.clone(),
        );

        // Handle by state machine
        let stateful = self.stateful_machine;
        let next_stateful = stateful.handle(data_access);
        {
          let mut state = lock!(self.state);
          state.update_state_machine(&next_stateful);
        }
        self.stateful_machine = next_stateful;

        // Exit loop and quit.
        if let StatefulValue::QuitState(_) = next_stateful {
          self.cancellation_token.cancel();
        }
      }
      Some(Err(e)) => {
        error!("Polled terminal event error: {:?}", e);
        self.cancellation_token.cancel();
      }
      None => {
        error!("Terminal event stream is exhausted, exit loop");
        self.cancellation_token.cancel();
      }
    }
  }

  async fn process_master_message(&mut self, message: Option<MasterMessage>) {
    if let Some(message) = message {
      match message {
        MasterMessage::PrintReq(req) => {
          trace!("Receive PrintReq:{:?}", req.future_id);
          let mut tree = lock!(self.tree);
          let mut contents = lock!(self.contents);
          cmdline_ops::cmdline_set_message(
            &mut tree,
            &mut contents,
            req.payload,
          );
        }
        MasterMessage::TimeoutReq(req) => {
          trace!("Receive TimeoutReq:{:?}", req.future_id);
          let jstick_tx = self.jstick_tx.clone();
          self.detached_tracker.spawn(async move {
            tokio::time::sleep(req.duration).await;
            let _ = jstick_tx
              .send(JsMessage::TimeoutResp(msg::TimeoutResp::new(
                req.future_id,
                req.duration,
              )))
              .await;
          });
        }
      }
    }
  }

  async fn process_jstick_message(&mut self, message: Option<JsMessage>) {
    if let Some(message) = message {
      trace!("Process resp msg:{:?}", message);
      let _ = self.jsrt_tx.send(message).await;
      self.js_runtime.tick_event_loop();
    }
  }

  async fn process_cancellation_notify(&mut self) {
    trace!("Receive cancellation token, exit loop");
    self.detached_tracker.close();
    self.blocked_tracker.close();
    self.blocked_tracker.wait().await;
  }

  /// Running the loop, it repeatedly do following steps:
  ///
  /// 1. Receives several things:
  ///    1. User keyboard/mouse events.
  ///    2. Received messages.
  ///    3. Cancellation request (which tells this event loop to quit).
  /// 2. Use the editing state (FSM) to handle the event.
  /// 3. Render the terminal.
  pub async fn run(&mut self) -> IoResult<()> {
    let mut reader = EventStream::new();
    loop {
      tokio::select! {
        // Receive keyboard/mouse events
        event = reader.next() => {
          self.process_event(event).await;
        }
        // Receive master message
        master_message = self.master_rx.recv() => {
            self.process_master_message(master_message).await;
        }
        // Receive loopback js message (should be sent to js runtime)
        js_resp = self.jstick_rx.recv() => {
            self.process_jstick_message(js_resp).await;
        }
        // Receive cancellation notify
        _ = self.cancellation_token.cancelled() => {
          self.process_cancellation_notify().await;
          break;
        }
      }

      // Flush logic UI to terminal, i.e. print UI to stdout
      lock!(self.tree).draw(self.canvas.clone());
      self.writer.write(&mut lock!(self.canvas))?;
    }

    Ok(())
  }

  #[cfg(test)]
  pub async fn mock_run(&mut self, mut reader: MockReader) -> IoResult<()> {
    loop {
      tokio::select! {
        // Receive mocked keyboard/mouse events
        event = reader.next() => {
          if is_ctrl_d(&event) {
            break;
          }
          self.process_event(event).await;
        }
        // Receive notification from js runtime => master
        js_req = self.master_rx.recv() => {
            self.process_master_message(js_req).await;
        }
        js_resp = self.jstick_rx.recv() => {
            self.process_jstick_message(js_resp).await;
        }
        // Receive cancellation notify
        _ = self.cancellation_token.cancelled() => {
          self.process_cancellation_notify().await;
          break;
        }
      }

      // Flush logic UI to terminal, i.e. print UI to stdout
      lock!(self.tree).draw(self.canvas.clone());
      self.writer.write(&mut lock!(self.canvas))?;
    }

    Ok(())
  }
}
