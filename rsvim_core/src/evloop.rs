//! Event loop.

use crate::buf::{BuffersManager, BuffersManagerArc};
use crate::cli::CliOpt;
use crate::envar;
use crate::evloop::msg::WorkerToMasterMessage;
use crate::js::msg::{self as jsmsg, EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::js::{JsRuntime, JsRuntimeOptions, SnapshotData};
use crate::lock;
use crate::prelude::*;
use crate::state::fsm::{StatefulDataAccess, StatefulValue, StatefulValueArc};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc, Shader, ShaderCommand};
use crate::ui::tree::*;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream,
};
use crossterm::{self, execute, queue};
use futures::StreamExt;
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
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

// #[derive(Debug)]
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

  /// Runtime path (directories). It initializes with following directories:
  ///
  /// 1. `$XDG_CONFIG_HOME/rsvim/` or `$HOME/.config/rsvim/`.
  /// 2. `$HOME/.rsvim/`
  ///
  /// Also see [`CONFIG_DIRS_PATH`](crate::envar::CONFIG_DIRS_PATH).
  ///
  /// NOTE: All the external plugins are been searched under runtime path.
  pub runtime_path: Arc<Mutex<Vec<PathBuf>>>,

  /// Widget tree for UI.
  pub tree: TreeArc,
  /// Canvas for UI.
  pub canvas: CanvasArc,
  /// Stdout writer for UI.
  pub writer: BufWriter<Stdout>,

  /// (Global) editing state.
  pub state: StateArc,

  /// Finite-state machine for editing state.
  pub stateful_machine: StatefulValueArc,

  /// Vim buffers.
  pub buffers: BuffersManagerArc,

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

  /// Sender: workers => master.
  ///
  /// NOTE: This sender stores here is mostly just for clone to all the other tasks spawned during
  /// running the editor. The master itself doesn't actually use it.
  pub worker_send_to_master: Sender<WorkerToMasterMessage>,
  /// Receiver: master <= workers.
  pub master_recv_from_worker: Receiver<WorkerToMasterMessage>,

  /// Js runtime.
  pub js_runtime: JsRuntime,
  /// Receiver: master <= js runtime.
  pub master_recv_from_js_runtime: Receiver<JsRuntimeToEventLoopMessage>,
  /// Sender: master => js runtime.
  pub master_send_to_js_runtime: Sender<EventLoopToJsRuntimeMessage>,
  /// An internal connected sender/receiver pair, it's simply for forward the task results
  /// to the event loop again and bypass the limitation of V8 engine.
  pub js_runtime_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>,
  pub js_runtime_tick_queue: Receiver<EventLoopToJsRuntimeMessage>,
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

    // State
    let state = State::to_arc(State::default());
    let stateful_machine = StatefulValue::to_arc(StatefulValue::default());

    // Worker => master
    let (worker_send_to_master, master_recv_from_worker) = channel(envar::CHANNEL_BUF_SIZE());

    // Since there are too many limitations that we cannot use tokio APIs along with V8 engine, we
    // have to first send task requests to master, let the master handles these tasks for us in the
    // async way, then send the task results back to js runtime.
    //
    // These tasks are very common and low level, serve as an infrastructure layer for js world.
    // For example:
    // - File IO
    // - Timer
    // - Network
    // - And more...
    //
    // The basic workflow is:
    // 1. When js runtime needs to handles the `Promise` and `async` functions, it send requests to
    //    master via `js_runtime_send_to_master`.
    // 2. Master receive requests via `master_recv_from_js_runtime`, and handle these tasks in async
    //    way.
    // 3. Master send the task results via `js_runtime_tick_dispatcher`.
    // 4. Master receive the task results via `js_runtime_tick_queue`, and send the results (again)
    //    via `master_send_to_js_runtime`.
    // 5. Js runtime receive these task results via `js_runtime_recv_from_master`, then process
    //    pending futures.
    //
    // You must notice that, the 3rd and 4th steps (and the pair of `js_runtime_tick_dispatcher`
    // and `js_runtime_tick_queue`) seem useless. Yes, they're simply for trigger the event loop
    // to run the `JsRuntime::tick_event_loop` API in `tokio::select!` main loop, due to the
    // limitation of V8 engine work along with tokio runtime.

    // Js runtime => master
    let (js_runtime_send_to_master, master_recv_from_js_runtime) =
      channel(envar::CHANNEL_BUF_SIZE());
    // Master => js runtime
    let (master_send_to_js_runtime, js_runtime_recv_from_master) =
      channel(envar::CHANNEL_BUF_SIZE());
    // Master => master
    let (js_runtime_tick_dispatcher, js_runtime_tick_queue) = channel(envar::CHANNEL_BUF_SIZE());

    // Runtime Path
    let runtime_path = envar::CONFIG_DIRS_PATH();
    let runtime_path = Arc::new(Mutex::new(runtime_path));

    // Task Tracker
    let detached_tracker = TaskTracker::new();
    let blocked_tracker = TaskTracker::new();
    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      snapshot,
      startup_moment,
      startup_unix_epoch,
      js_runtime_send_to_master,
      js_runtime_recv_from_master,
      cli_opt.clone(),
      runtime_path.clone(),
      tree.clone(),
      buffers_manager.clone(),
      state.clone(),
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opt,
      runtime_path,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers: buffers_manager,
      writer: BufWriter::new(std::io::stdout()),
      cancellation_token: CancellationToken::new(),
      detached_tracker,
      blocked_tracker,
      worker_send_to_master,
      master_recv_from_worker,
      js_runtime,
      master_recv_from_js_runtime,
      master_send_to_js_runtime,
      js_runtime_tick_dispatcher,
      js_runtime_tick_queue,
    })
  }

  /// Initialize user config file.
  pub fn init_config(&mut self) -> IoResult<()> {
    if let Some(config_file) = envar::CONFIG_FILE_PATH() {
      self
        .js_runtime
        .execute_module(config_file.to_str().unwrap(), None)
        .unwrap();
    }
    Ok(())
  }

  /// Initialize terminal raw mode.
  pub fn init_tui(&self) -> IoResult<()> {
    if !crossterm::terminal::is_raw_mode_enabled()? {
      crossterm::terminal::enable_raw_mode()?;
    }

    let mut out = std::io::stdout();
    execute!(
      out,
      crossterm::terminal::EnterAlternateScreen,
      crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
      EnableMouseCapture,
      EnableFocusChange,
    )?;

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
    let mut out = std::io::stdout();
    execute!(
      out,
      DisableMouseCapture,
      DisableFocusChange,
      crossterm::terminal::LeaveAlternateScreen,
    )?;

    if crossterm::terminal::is_raw_mode_enabled()? {
      crossterm::terminal::disable_raw_mode()?;
    }

    Ok(())
  }

  /// Initialize buffers.
  pub fn init_buffers(&mut self) -> IoResult<()> {
    let canvas_size = lock!(self.canvas).size();

    // Create buffers from parameters.
    let input_files = self.cli_opt.file().to_vec();
    if !input_files.is_empty() {
      for input_file in input_files.iter() {
        let maybe_buf_id =
          lock!(self.buffers).new_file_buffer(canvas_size.height(), Path::new(input_file));
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
      let buf_id = lock!(self.buffers).new_empty_buffer(canvas_size.height());
      trace!("Created empty buffer {:?}", buf_id);
    }

    Ok(())
  }

  /// Initialize windows.
  pub fn init_windows(&mut self) -> IoResult<()> {
    // Initialize default window.
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
      (canvas_size.width() as isize, canvas_size.height() as isize),
    );
    let window = {
      let buffers = lock!(self.buffers);
      let (buf_id, buf) = buffers.first_key_value().unwrap();
      trace!("Bind first buffer to default window {:?}", buf_id);
      Window::new(
        window_shape,
        Arc::downgrade(buf),
        tree.global_local_options(),
      )
    };
    let window_id = window.id();
    let window_node = TreeNode::Window(window);
    tree.bounded_insert(tree_root_id, window_node);

    // Initialize cursor.
    let cursor_shape = IRect::new((0, 0), (1, 1));
    let cursor = Cursor::new(
      cursor_shape,
      canvas_cursor.blinking(),
      canvas_cursor.hidden(),
      canvas_cursor.style(),
    );
    let cursor_node = TreeNode::Cursor(cursor);
    tree.bounded_insert(window_id, cursor_node);

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
          event,
        );

        // Handle by state machine
        let next_stateful = self.stateful_machine.clone().handle(data_access);
        let next_stateful = StatefulValue::to_arc(next_stateful);
        {
          let mut state = lock!(self.state);
          state.update_state_machine(&next_stateful);
        }
        self.stateful_machine = next_stateful.clone();

        // Exit loop and quit.
        if let StatefulValue::QuitState(_) = *next_stateful {
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
          trace!("process_js_runtime_request timeout_req:{:?}", req.future_id);
          let js_runtime_tick_dispatcher = self.js_runtime_tick_dispatcher.clone();
          self.detached_tracker.spawn(async move {
            tokio::time::sleep(req.duration).await;
            let _ = js_runtime_tick_dispatcher
              .send(EventLoopToJsRuntimeMessage::TimeoutResp(
                jsmsg::TimeoutResp::new(req.future_id, req.duration),
              ))
              .await;
            trace!(
              "process_js_runtime_request timeout_req:{:?} - done",
              req.future_id
            );
          });
        }
      }
    }
  }

  async fn process_js_runtime_response(&mut self, msg: Option<EventLoopToJsRuntimeMessage>) {
    if let Some(msg) = msg {
      trace!("process_js_runtime_response msg:{:?}", msg);
      let _ = self.master_send_to_js_runtime.send(msg).await;
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
        // Receive notification from workers
        worker_msg = self.master_recv_from_worker.recv() => {
          self.process_worker_notify(worker_msg).await;
        }
        // Receive notification from js runtime
        js_req = self.master_recv_from_js_runtime.recv() => {
            self.process_js_runtime_request(js_req).await;
        }
        js_resp = self.js_runtime_tick_queue.recv() => {
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
