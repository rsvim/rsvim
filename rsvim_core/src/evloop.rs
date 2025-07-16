//! Event loop.

use crate::buf::{BuffersManager, BuffersManagerArc};
use crate::cli::CliOpt;
use crate::consts;
use crate::content::{TextContents, TextContentsArc};
use crate::evloop::msg::WorkerToMasterMessage;
use crate::js::msg::{self as jsmsg, EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::js::{JsRuntime, JsRuntimeOptions, SnapshotData};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc, Shader, ShaderCommand};
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use crossterm::event::{Event, EventStream};
use crossterm::{self, queue};
use futures::StreamExt;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
// use heed::types::U16;
use std::io::Write;
use std::io::{BufWriter, Stdout};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{error, trace};

pub mod msg;
pub mod task;
pub mod tui;

#[derive(Debug)]
/// For slow tasks that are suitable to put in the background, this event loop will spawn them in
/// tokio's async tasks and let them sync back data once they are done. The event loop controls all
/// the tasks with [`CancellationToken`] and [`TaskTracker`].
///
/// # Terms
///
/// * Master: The event loop itself.
/// * Worker: A spawned task.
/// * JsRuntime: The javascript runtime (including V8 engine).
///
/// Js runtime and this event loop communicate via another two pairs of channels.
pub struct EventLoop {
  /// Indicates the start time of the process.
  pub startup_moment: Instant,
  /// Specifies the timestamp which the current process began in Unix time.
  pub startup_unix_epoch: u128,

  /// Command line options.
  pub cli_opt: CliOpt,

  /// Widget tree for UI.
  pub tree: TreeArc,
  /// Canvas for UI.
  pub canvas: CanvasArc,
  /// Stdout writer for UI.
  pub writer: BufWriter<Stdout>,

  /// (Global) editing state.
  pub state: StateArc,

  /// Finite-state machine for editing state.
  pub stateful_machine: StatefulValue,

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

  /// Channel: "workers" => "master"
  /// NOTE: In variables naming, we use "wkr" for "workers", "mstr" for "master".
  ///
  /// Sender: workers send to master.
  pub wkr_to_mstr: Sender<WorkerToMasterMessage>,
  /// Receiver: master receive from workers.
  pub mstr_from_wkr: Receiver<WorkerToMasterMessage>,

  /// Js runtime.
  pub js_runtime: JsRuntime,

  /// Channel: "master" => "js runtime"
  /// NOTE: In variables naming, we use "mstr" for "master", "jsrt" for "js runtime".
  ///
  /// Receiver: master receive from js runtime.
  pub mstr_from_jsrt: Receiver<JsRuntimeToEventLoopMessage>,
  /// Sender: master send to js runtime.
  pub mstr_to_jsrt: Sender<EventLoopToJsRuntimeMessage>,

  /// Channel: "master" => "master" ("dispatcher" => "queue")
  ///
  /// Sender: dispatcher.
  pub jsrt_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>,
  /// Receiver: queue.
  pub jsrt_tick_queue: Receiver<EventLoopToJsRuntimeMessage>,
}

impl EventLoop {
  /// Make new event loop.
  pub fn new(cli_opt: CliOpt, snapshot: SnapshotData) -> IoResult<Self> {
    // Canvas
    let (cols, rows) = crossterm::terminal::size()?;
    let canvas_size = U16Size::new(cols, rows);
    let canvas = Canvas::new(canvas_size);
    let canvas = Canvas::to_arc(canvas);

    // UI Tree
    let tree = Tree::to_arc(Tree::new(canvas_size));

    // Buffers
    let buffers_manager = BuffersManager::to_arc(BuffersManager::new());
    let text_contents = TextContents::to_arc(TextContents::new(canvas_size));

    // Channel: workers => master
    let (wkr_to_mstr, mstr_from_wkr) = channel(*consts::CHANNEL_BUF_SIZE);

    // Since there are technical limitations that we cannot use tokio APIs along with V8 engine,
    // because V8 rust bindings are not Arc/Mutex (i.e. not thread safe), while tokio async runtime
    // requires Arc/Mutex (i.e. thread safe).
    //
    // We have to first send js task requests to master, let the master handles these tasks for us
    // (in async way), then send the task results back to js runtime. These tasks are very common
    // and low level, serve as an infrastructure layer for js world.
    // For example:
    // - File IO
    // - Timer
    // - Network
    // - And more...
    //
    // When js runtime handles `Promise` and `async` APIs, the message flow uses several channels:
    //
    // - Channel-1 `jsrt_to_mstr` => `mstr_from_jsrt`, on message `JsRuntimeToEventLoopMessage`.
    // - Channel-2 `jsrt_tick_dispatcher` => `jsrt_tick_queue`, on message `EventLoopToJsRuntimeMessage`.
    // - Channel-3 `mstr_to_jsrt` => `jsrt_from_mstr`, on message `EventLoopToJsRuntimeMessage`.
    //
    // ```text
    //
    // Step-1: Js runtime --- JsRuntimeToEventLoopMessage (channel-1) --> Tokio event loop
    // Step-2: Tokio event loop handles the request (read/write, timer, etc) in async way
    // Step-3: Tokio event loop --- EventLoopToJsRuntimeMessage (channel-2) --> Tokio event loop
    // Step-4: Tokio event loop --- EventLoopToJsRuntimeMessage (channel-3) --> Js runtime
    //
    // ```
    //
    // NOTE: You must notice, the step-3 and channel-2 seems unnecessary. Yes, they're simply for
    // trigger the event loop in `tokio::select!`.

    // Channel: js runtime => master
    let (jsrt_to_mstr, mstr_from_jsrt) = channel(*consts::CHANNEL_BUF_SIZE);
    // Channel: master => js runtime
    let (mstr_to_jsrt, jsrt_from_mstr) = channel(*consts::CHANNEL_BUF_SIZE);
    // Channel: master => master
    let (jsrt_tick_dispatcher, jsrt_tick_queue) = channel(*consts::CHANNEL_BUF_SIZE);

    // Task Tracker
    let detached_tracker = TaskTracker::new();
    let blocked_tracker = TaskTracker::new();
    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    // State
    let state = State::to_arc(State::new(jsrt_tick_dispatcher.clone()));
    let stateful_machine = StatefulValue::default();

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      snapshot,
      startup_moment,
      startup_unix_epoch,
      jsrt_to_mstr,
      jsrt_from_mstr,
      cli_opt.clone(),
      tree.clone(),
      buffers_manager.clone(),
      text_contents.clone(),
      state.clone(),
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opt,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers: buffers_manager,
      contents: text_contents,
      writer: BufWriter::new(std::io::stdout()),
      cancellation_token: CancellationToken::new(),
      detached_tracker,
      blocked_tracker,
      wkr_to_mstr,
      mstr_from_wkr,
      js_runtime,
      mstr_from_jsrt,
      mstr_to_jsrt,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
    })
  }

  /// Initialize user config file.
  pub fn init_config(&mut self) -> IoResult<()> {
    if let Some(config_entry) = &*consts::CONFIG_ENTRY_PATH {
      self
        .js_runtime
        .execute_module(config_entry.to_str().unwrap(), None)
        .unwrap();
    }
    Ok(())
  }

  /// Initialize terminal raw mode.
  pub fn init_tui(&self) -> IoResult<()> {
    tui::initialize_raw_mode()?;

    // Register panic hook to shutdown terminal raw mode, this helps recover normal terminal
    // command line for users, if any exceptions been thrown.
    tui::shutdown_raw_mode_on_panic();

    Ok(())
  }

  /// First flush TUI to terminal.
  pub fn init_tui_complete(&mut self) -> IoResult<()> {
    // Initialize cursor
    let cursor = {
      let canvas = lock!(self.canvas);
      *canvas.frame().cursor()
    };

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

    self.render()?;

    Ok(())
  }

  /// Shutdown terminal raw mode.
  pub fn shutdown_tui(&self) -> IoResult<()> {
    tui::shutdown_raw_mode()
  }

  /// Initialize buffers.
  pub fn init_buffers(&mut self) -> IoResult<()> {
    let canvas_size = lock!(self.canvas).size();

    // Create default buffer from `FILES` arguments from cli, or with an empty buffer.
    let input_files = self.cli_opt.file().to_vec();
    if !input_files.is_empty() {
      for input_file in input_files.iter() {
        let maybe_buf_id = lock!(self.buffers).new_file_buffer(canvas_size, Path::new(input_file));
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
  pub fn init_windows(&mut self) -> IoResult<()> {
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
    let cmdline = CommandLine::new(cmdline_shape, Arc::downgrade(&self.contents));
    let _cmdline_id = cmdline.id();

    tree.bounded_insert(tree_root_id, TreeNode::CommandLine(cmdline));

    Ok(())
  }

  async fn process_event(&mut self, event: Option<IoResult<Event>>) {
    match event {
      Some(Ok(event)) => {
        trace!("Polled terminal event ok: {:?}", event);

        let data_access = StatefulDataAccess::new(
          self.state.clone(),
          self.tree.clone(),
          self.buffers.clone(),
          self.contents.clone(),
          event,
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

  async fn process_worker_notify(&mut self, msg: Option<WorkerToMasterMessage>) {
    trace!("Received {:?} message from workers", msg);
  }

  async fn process_js_runtime_request(&mut self, msg: Option<JsRuntimeToEventLoopMessage>) {
    if let Some(msg) = msg {
      match msg {
        JsRuntimeToEventLoopMessage::TimeoutReq(req) => {
          trace!("Receive req timeout_req:{:?}", req.future_id);
          let jsrt_tick_dispatcher = self.jsrt_tick_dispatcher.clone();
          self.detached_tracker.spawn(async move {
            tokio::time::sleep(req.duration).await;
            let _ = jsrt_tick_dispatcher
              .send(EventLoopToJsRuntimeMessage::TimeoutResp(
                jsmsg::TimeoutResp::new(req.future_id, req.duration),
              ))
              .await;
            trace!("Receive req timeout_req:{:?} - done", req.future_id);
          });
        }
      }
    }
  }

  async fn process_js_runtime_response(&mut self, msg: Option<EventLoopToJsRuntimeMessage>) {
    if let Some(msg) = msg {
      trace!("Process resp msg:{:?}", msg);
      let _ = self.mstr_to_jsrt.send(msg).await;
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
  ///    2. Messages sent from workers.
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
        // Receive notification from workers => master
        worker_msg = self.mstr_from_wkr.recv() => {
          self.process_worker_notify(worker_msg).await;
        }
        // Receive notification from js runtime => master
        js_req = self.mstr_from_jsrt.recv() => {
            self.process_js_runtime_request(js_req).await;
        }
        js_resp = self.jsrt_tick_queue.recv() => {
            self.process_js_runtime_response(js_resp).await;
        }
        // Receive cancellation notify
        _ = self.cancellation_token.cancelled() => {
          self.process_cancellation_notify().await;
          // let _ = self.master_send_to_js_worker.send(EventLoopToJsRuntimeMessage::Shutdown(jsmsg::Dummy::default())).await;
          break;
        }
      }

      // Update terminal
      self.render()?;
    }

    Ok(())
  }

  fn render(&mut self) -> IoResult<()> {
    // Draw UI components to the canvas.
    lock!(self.tree).draw(self.canvas.clone());

    // Compute the commands that need to output to the terminal device.
    let shader = lock!(self.canvas).shade();

    self.queue_shader(shader)?;
    self.writer.flush()?;

    Ok(())
  }

  /// Put (render) canvas shader.
  fn queue_shader(&mut self, shader: Shader) -> IoResult<()> {
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
}
