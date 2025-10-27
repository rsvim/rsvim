//! Event loop.

pub mod writer;

use crate::buf::BuffersManager;
use crate::buf::BuffersManagerArc;
use crate::cli::CliOptions;
use crate::content::TextContents;
use crate::content::TextContentsArc;
use crate::js::JsRuntime;
use crate::js::JsRuntimeOptions;
use crate::js::SnapshotData;
use crate::js::binding::global_rsvim::fs::open::async_fs_open;
use crate::js::binding::global_rsvim::fs::read::async_fs_read;
use crate::js::command::CommandsManager;
use crate::js::command::CommandsManagerArc;
use crate::js::encdec::encode_bytes;
use crate::js::module::async_load_import;
use crate::msg;
use crate::msg::JsMessage;
use crate::msg::MasterMessage;
use crate::prelude::*;
use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::ops::cmdline_ops;
use crate::ui::canvas::Canvas;
use crate::ui::canvas::CanvasArc;
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;
use crossterm::event::Event;
use crossterm::event::EventStream;
use futures::StreamExt;
use ringbuf::traits::RingBuffer;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use writer::StdoutWritable;
use writer::StdoutWriterValue;

#[cfg(test)]
use crate::tests::evloop::MockEventReader;
#[cfg(test)]
use crate::tests::evloop::MockOperation;
#[cfg(test)]
use crate::tests::evloop::MockOperationReader;
#[cfg(test)]
use bitflags::bitflags_match;
#[cfg(test)]
use crossterm::event::KeyCode;
#[cfg(test)]
use crossterm::event::KeyEventKind;
#[cfg(test)]
use crossterm::event::KeyModifiers;

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

  /// Finite-state machine for editing state.
  pub state_machine: StateMachine,

  /// Vim buffers.
  pub buffers: BuffersManagerArc,
  /// Text contents (except buffers).
  pub contents: TextContentsArc,

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
  pub exit_code: i32,

  /// Js runtime.
  pub js_runtime: JsRuntime,

  /// Channel-1
  pub master_tx: UnboundedSender<MasterMessage>,
  pub master_rx: UnboundedReceiver<MasterMessage>,

  /// Channel-2
  pub jsrt_forwarder_tx: UnboundedSender<JsMessage>,
  pub jsrt_forwarder_rx: UnboundedReceiver<JsMessage>,

  /// Channel-3
  pub jsrt_tx: UnboundedSender<JsMessage>,
  // pub jsrt_rx: UnboundedReceiver<JsMessage>,
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
    /* canvas */ CanvasArc,
    /* tree */ TreeArc,
    /* state_machine */ StateMachine,
    /* buffers */ BuffersManagerArc,
    /* contents */ TextContentsArc,
    /* commands */ CommandsManagerArc,
    /* cancellation_token */ CancellationToken,
    /* detached_tracker */ TaskTracker,
    /* blocked_tracker */ TaskTracker,
    /* exit_code */ i32,
    (
      /* master_tx */ UnboundedSender<MasterMessage>,
      /* master_rx */ UnboundedReceiver<MasterMessage>,
    ),
    (
      /* jsrt_forwarder_tx */ UnboundedSender<JsMessage>,
      /* jsrt_forwarder_rx */ UnboundedReceiver<JsMessage>,
    ),
    (
      /* jsrt_tx */ UnboundedSender<JsMessage>,
      /* jsrt_rx */ UnboundedReceiver<JsMessage>,
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
      CommandsManager::to_arc(CommandsManager::default());

    // State
    let state_machine = StateMachine::default();

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
    // - Channel-2 `jsrt_forwarder_tx` => `jsrt_forwarder_rx` on `JsMessage`.
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

    // Channel-1
    let (master_tx, master_rx) = unbounded_channel();
    // Channel-2
    let (jsrt_forwarder_tx, jsrt_forwarder_rx) = unbounded_channel();
    // Channel-3
    let (jsrt_tx, jsrt_rx) = unbounded_channel();

    Ok((
      canvas,
      tree,
      state_machine,
      buffers_manager,
      text_contents,
      ex_commands_manager,
      CancellationToken::new(),
      TaskTracker::new(),
      TaskTracker::new(),
      0,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
      (jsrt_tx, jsrt_rx),
    ))
  }

  /// Make new event loop.
  pub fn new(
    startup_moment: Instant,
    startup_unix_epoch: u128,
    cli_opts: CliOptions,
    snapshot: SnapshotData,
  ) -> IoResult<Self> {
    let (cols, rows) = crossterm::terminal::size()?;
    let (
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
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
      commands,
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      js_runtime,
      master_tx,
      master_rx,
      jsrt_forwarder_tx,
      jsrt_forwarder_rx,
      jsrt_tx,
    })
  }

  #[cfg(test)]
  /// Make new event loop for testing.
  pub fn mock_new_without_snapshot(
    terminal_columns: u16,
    terminal_rows: u16,
    cli_opts: CliOptions,
  ) -> IoResult<Self> {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    let (
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
      (jsrt_tx, jsrt_rx),
    ) = Self::_internal_new(terminal_columns, terminal_rows)?;

    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();
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
      commands,
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      js_runtime,
      master_tx,
      master_rx,
      jsrt_forwarder_tx,
      jsrt_forwarder_rx,
      jsrt_tx,
    })
  }

  #[cfg(test)]
  /// Make new event loop for testing.
  pub fn mock_new_with_snapshot(
    terminal_columns: u16,
    terminal_rows: u16,
    cli_opts: CliOptions,
    snapshot: SnapshotData,
  ) -> IoResult<Self> {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    let (
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      commands,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      (master_tx, master_rx),
      (jsrt_forwarder_tx, jsrt_forwarder_rx),
      (jsrt_tx, jsrt_rx),
    ) = Self::_internal_new(terminal_columns, terminal_rows)?;

    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let writer = StdoutWriterValue::dev_null();

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
      commands,
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opts,
      canvas,
      tree,
      state_machine,
      buffers,
      contents,
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      exit_code,
      js_runtime,
      master_tx,
      master_rx,
      jsrt_forwarder_tx,
      jsrt_forwarder_rx,
      jsrt_tx,
    })
  }

  /// Initialize the editor
  pub fn initialize(&mut self) -> IoResult<()> {
    // Initialize user js configs
    self._init_config()?;

    self.writer.init()?;

    // Initialize TUI
    self._init_buffers()?;
    self._init_windows()?;
    self._init_pending_messages();

    // Flush logic UI to terminal, i.e. print UI to stdout
    lock!(self.tree).draw(self.canvas.clone());
    self.writer.init_complete(&mut lock!(self.canvas))?;

    Ok(())
  }

  /// Initialize user config file.
  fn _init_config(&mut self) -> IoResult<()> {
    if let Some(config_entry) = PATH_CONFIG.config_entry() {
      self
        .js_runtime
        .execute_module(&config_entry.to_string_lossy(), None);

      // Extra tick for some pending imports.
      self.js_runtime.tick_event_loop();
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
            // Send error message to command-line
            error!("Failed to create file buffer {:?}:{:?}", input_file, e);

            // Append error message to command line message history, wait for
            // print once TUI initialized.
            let mut contents = lock!(self.contents);
            contents
              .command_line_message_history_mut()
              .push_overwrite(e.to_string());
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

  // Since we run user's config script (i.e. `.rsvim/rsvim.js`) before
  // initializing TUI/UI-tree. If user calls the `Rsvim.cmd.echo` API directly
  // in their configs right before the editor TUI initialize, the UI tree is
  // not created, and the "command-line-message" widget inside UI tree does not
  // exist.
  //
  // Thus we will have to store the printed messages in
  // `contents.command_line_message_history` temporarily. If the messages are
  // just too many, old messages will be thrown, only new messages are left.
  //
  // And all messages will be print once the editor TUI is initialized.
  fn _init_pending_messages(&mut self) {
    let mut contents = lock!(self.contents);
    let mut tree = lock!(self.tree);
    cmdline_ops::cmdline_flush_pending_message(&mut tree, &mut contents);
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
        let data_access = StateDataAccess::new(
          self.tree.clone(),
          self.buffers.clone(),
          self.contents.clone(),
          self.master_tx.clone(),
          self.jsrt_forwarder_tx.clone(),
        );

        // Handle by state machine
        let stateful = self.state_machine;
        let next_stateful = stateful.handle(data_access, event);
        self.state_machine = next_stateful;
      }
      Some(Err(e)) => {
        error!("Polled terminal event error: {:?}", e);
        // self.cancellation_token.cancel();
      }
      None => {
        error!("Terminal event stream is exhausted");
        // self.cancellation_token.cancel();
      }
    }
  }

  #[cfg(test)]
  async fn _process_mocked_operations(
    &mut self,
    op: Option<IoResult<MockOperation>>,
  ) {
    match op {
      Some(Ok(op)) => {
        trace!("Polled editor operation ok: {:?}", op);
        match op {
          MockOperation::Operation(op) => {
            let data_access = StateDataAccess::new(
              self.tree.clone(),
              self.buffers.clone(),
              self.contents.clone(),
              self.master_tx.clone(),
              self.jsrt_forwarder_tx.clone(),
            );

            // Handle by state machine
            let stateful = self.state_machine;
            let next_stateful = stateful.handle_op(data_access, op);
            self.state_machine = next_stateful;
          }
          MockOperation::Exit => {
            self.cancellation_token.cancel();
          }
          _ => unreachable!(),
        }
      }
      Some(Err(e)) => {
        error!("Polled terminal event error: {:?}", e);
        // self.cancellation_token.cancel();
      }
      None => {
        error!("Terminal event stream is exhausted");
        // self.cancellation_token.cancel();
      }
    }
  }

  async fn process_master_message(&mut self, message: Option<MasterMessage>) {
    if let Some(message) = message {
      match message {
        MasterMessage::ExitReq(req) => {
          trace!("Recv ExitReq:{:?}", req.exit_code);
          self.exit_code = req.exit_code;
          self.cancellation_token.cancel();
        }
        MasterMessage::TimeoutReq(req) => {
          trace!("Recv TimeoutReq:{:?}", req.timer_id);
          let jsrt_forwarder_tx = self.jsrt_forwarder_tx.clone();
          self.detached_tracker.spawn(async move {
            let expire_at = req.start_at
              + tokio::time::Duration::from_millis(req.delay as u64);
            tokio::time::sleep_until(expire_at).await;
            jsrt_forwarder_tx
              .send(JsMessage::TimeoutResp(msg::TimeoutResp {
                timer_id: req.timer_id,
                expire_at,
                delay: req.delay,
                repeated: req.repeated,
              }))
              .unwrap();
          });
        }
        MasterMessage::LoadImportReq(req) => {
          trace!("Recv LoadImportReq:{:?}", req.task_id);
          let jsrt_forwarder_tx = self.jsrt_forwarder_tx.clone();
          self.detached_tracker.spawn(async move {
            let maybe_source = async_load_import(&req.specifier, false).await;
            jsrt_forwarder_tx
              .send(JsMessage::LoadImportResp(msg::LoadImportResp {
                task_id: req.task_id,
                maybe_source: match maybe_source {
                  Ok(source) => Some(Ok(encode_bytes(source))),
                  Err(e) => Some(Err(e)),
                },
              }))
              .unwrap();
          });
        }
        MasterMessage::TickAgainReq => {
          trace!("Recv TickAgainReq");
          let jsrt_forwarder_tx = self.jsrt_forwarder_tx.clone();
          self.detached_tracker.spawn(async move {
            jsrt_forwarder_tx.send(JsMessage::TickAgainResp).unwrap();
          });
        }
        MasterMessage::FsOpenReq(req) => {
          trace!("Recv FsOpenReq");
          let jsrt_forwarder_tx = self.jsrt_forwarder_tx.clone();
          self.detached_tracker.spawn(async move {
            let maybe_result = async_fs_open(&req.path, req.options).await;
            jsrt_forwarder_tx
              .send(JsMessage::FsOpenResp(msg::FsOpenResp {
                task_id: req.task_id,
                maybe_result: match maybe_result {
                  Ok(fd) => Some(Ok(encode_bytes(fd))),
                  Err(e) => Some(Err(e)),
                },
              }))
              .unwrap();
          });
        }
        MasterMessage::FsReadReq(req) => {
          trace!("Recv FsReadReq");
          let jsrt_forwarder_tx = self.jsrt_forwarder_tx.clone();
          self.detached_tracker.spawn(async move {
            let maybe_result = async_fs_read(req.fd, req.bufsize).await;
            jsrt_forwarder_tx
              .send(JsMessage::FsOpenResp(msg::FsOpenResp {
                task_id: req.task_id,
                maybe_result: match maybe_result {
                  Ok(fd) => Some(Ok(encode_bytes(fd))),
                  Err(e) => Some(Err(e)),
                },
              }))
              .unwrap();
          });
        }
      }
    }
  }

  async fn forward_js_message(&mut self, message: Option<JsMessage>) {
    if let Some(message) = message {
      trace!("Process resp msg:{:?}", message);
      self.jsrt_tx.send(message).unwrap();
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
        master_msg = self.master_rx.recv() => {
          self.process_master_message(master_msg).await;
        }
        // Receive loopback js message (should be sent to js runtime)
        js_msg = self.jsrt_forwarder_rx.recv() => {
          self.forward_js_message(js_msg).await;
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
  pub async fn run_with_mock_events(
    &mut self,
    mut reader: MockEventReader,
  ) -> IoResult<()> {
    loop {
      tokio::select! {
        // Receive mocked keyboard/mouse events
        event = reader.next() => {
          if is_ctrl_d(&event) {
            break;
          }
          self.process_event(event).await;
        }
        master_msg = self.master_rx.recv() => {
          self.process_master_message(master_msg).await;
        }
        js_msg = self.jsrt_forwarder_rx.recv() => {
          self.forward_js_message(js_msg).await;
        }
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
  pub async fn run_with_mock_operations(
    &mut self,
    mut reader: MockOperationReader,
  ) -> IoResult<()> {
    loop {
      tokio::select! {
        // Receive mocked keyboard/mouse events
        op = reader.next() => {
          self._process_mocked_operations(op).await;
        }
        master_msg = self.master_rx.recv() => {
          self.process_master_message(master_msg).await;
        }
        js_msg = self.jsrt_forwarder_rx.recv() => {
          self.forward_js_message(js_msg).await;
        }
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
