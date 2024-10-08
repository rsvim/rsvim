//! Main event loop.

#![allow(dead_code)]

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream,
};
use crossterm::{self, execute, queue};
use futures::StreamExt;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
// use heed::types::U16;
use std::io::Write;
use std::io::{BufWriter, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error};

use crate::buf::{Buffer, Buffers, BuffersArc};
use crate::cart::{IRect, U16Size};
use crate::cli::CliOpt;
use crate::evloop::msg::WorkerToMasterMessage;
use crate::evloop::task::TaskableDataAccess;
use crate::glovar;
use crate::js::msg::{self as jsmsg, EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage};
use crate::js::{JsRuntime, JsRuntimeOptions};
use crate::result::{IoResult, VoidIoResult};
use crate::state::fsm::StatefulValue;
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc, Shader, ShaderCommand};
use crate::ui::tree::internal::Inodeable;
use crate::ui::tree::{Tree, TreeArc, TreeNode};
use crate::ui::widget::{Cursor, Window};

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
  /// 1. [`CONFIG_FILE_PATH`](crate::glovar::CONFIG_FILE_PATH)
  /// 2. [`DATA_DIR_PATH`](crate::glovar::DATA_DIR_PATH)
  ///
  /// NOTE: All the external plugins are been searched under runtime path.
  pub runtime_path: Arc<RwLock<Vec<PathBuf>>>,

  /// Widget tree for UI.
  pub tree: TreeArc,
  /// Canvas for UI.
  pub canvas: CanvasArc,
  /// Stdout writer for UI.
  pub writer: BufWriter<Stdout>,

  /// (Global) editing state.
  pub state: StateArc,

  /// Vim buffers.
  pub buffers: BuffersArc,

  /// Cancellation token to notify the main loop to exit.
  pub cancellation_token: CancellationToken,
  /// Task tracker for all spawned tasks.
  pub task_tracker: TaskTracker,

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
  pub fn new(cli_opt: CliOpt) -> IoResult<Self> {
    // Canvas
    let (cols, rows) = crossterm::terminal::size()?;
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
    let buffers_arc = Buffers::to_arc(buffers);

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
    let tree_arc = Tree::to_arc(tree);

    // State
    let state = State::default();
    let state_arc = State::to_arc(state);

    // Worker => master
    let (worker_send_to_master, master_recv_from_worker) = channel(glovar::CHANNEL_BUF_SIZE());

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
      channel(glovar::CHANNEL_BUF_SIZE());
    // Master => js runtime
    let (master_send_to_js_runtime, js_runtime_recv_from_master) =
      channel(glovar::CHANNEL_BUF_SIZE());
    // Master => master
    let (js_runtime_tick_dispatcher, js_runtime_tick_queue) = channel(glovar::CHANNEL_BUF_SIZE());

    // Runtime Path
    let mut runtime_path = vec![glovar::DATA_DIR_PATH()];
    if glovar::CONFIG_FILE_PATH().is_some() {
      runtime_path.push(glovar::CONFIG_FILE_PATH().unwrap());
    }
    let runtime_path = Arc::new(RwLock::new(runtime_path));

    // Task Tracker
    let task_tracker = TaskTracker::new();
    let task_local_set = LocalSet::new();
    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      startup_moment,
      startup_unix_epoch,
      task_local_set,
      js_runtime_send_to_master,
      js_runtime_recv_from_master,
      cli_opt.clone(),
      runtime_path.clone(),
      tree_arc.clone(),
      buffers_arc.clone(),
      state_arc.clone(),
    );

    Ok(EventLoop {
      startup_moment,
      startup_unix_epoch,
      cli_opt,
      runtime_path,
      canvas,
      tree: tree_arc,
      state: state_arc,
      buffers: buffers_arc,
      writer: BufWriter::new(std::io::stdout()),
      cancellation_token: CancellationToken::new(),
      task_tracker,
      worker_send_to_master,
      master_recv_from_worker,
      js_runtime,
      master_recv_from_js_runtime,
      master_send_to_js_runtime,
      js_runtime_tick_dispatcher,
      js_runtime_tick_queue,
    })
  }

  /// Initialize TUI.
  pub fn init_tui(&self) -> VoidIoResult {
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

  /// Initialize js runtime.
  pub fn init_js_runtime(&mut self) -> VoidIoResult {
    self.js_runtime.init_environment();
    if let Some(config_file) = glovar::CONFIG_FILE_PATH() {
      self
        .js_runtime
        .execute_module(config_file.to_str().unwrap(), None)
        .unwrap();
    }
    Ok(())
  }

  /// Initialize start up tasks such as input files, etc.
  pub fn init_input_files(&mut self) -> VoidIoResult {
    self.queue_cursor()?;
    self.writer.flush()?;

    // // Register fixed FPS loop to update self (to update the terminal), thus avoid duplicated
    // // messages sent from js runtime side, because our channels are fixed sized.
    // {
    //   let data_access = TaskableDataAccess::new(
    //     self.state.clone(),
    //     self.tree.clone(),
    //     self.buffers.clone(),
    //     self.worker_send_to_master.clone(),
    //   );
    //   self.task_tracker.spawn(async move {
    //     let _ = task::startup::fixed_rate_update::update_in_fixed_rate(
    //       data_access,
    //       glovar::FIXED_RATE_UPDATE_MILLIS(),
    //     )
    //     .await;
    //   });
    // }

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

  async fn process_event(&mut self, next_event: Option<IoResult<Event>>) {
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
    debug!("Received {:?} message from workers", msg);
  }

  async fn process_js_runtime_request(&mut self, msg: Option<JsRuntimeToEventLoopMessage>) {
    if let Some(msg) = msg {
      match msg {
        JsRuntimeToEventLoopMessage::TimeoutReq(req) => {
          debug!("process_js_runtime_request timeout_req:{:?}", req.future_id);
          let js_runtime_tick_dispatcher = self.js_runtime_tick_dispatcher.clone();
          self.task_tracker.spawn(async move {
            tokio::time::sleep(req.duration).await;
            let _ = js_runtime_tick_dispatcher
              .send(EventLoopToJsRuntimeMessage::TimeoutResp(
                jsmsg::TimeoutResp::new(req.future_id, req.duration),
              ))
              .await;
            debug!(
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
      debug!("process_js_runtime_response msg:{:?}", msg);
      let _ = self.master_send_to_js_runtime.send(msg).await;
      self.js_runtime.tick_event_loop();
    }
  }

  /// Running the loop, it repeatedly do following steps:
  ///
  /// 1. Receives several things:
  ///    1. User keyboard/mouse events.
  ///    2. Messages sent from workers.
  ///    3. Cancellation request (which tells this event loop to quit).
  /// 2. Use the editing state (FSM) to handle the event.
  /// 3. Render the terminal.
  pub async fn run(&mut self) -> VoidIoResult {
    let mut reader = EventStream::new();
    loop {
      tokio::select! {
        // Receive keyboard/mouse events
        next_event = reader.next() => {
          self.process_event(next_event).await;
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
          debug!("Receive cancellation token, exit loop");
          self.task_tracker.close();
          // let _ = self.master_send_to_js_worker.send(EventLoopToJsRuntimeMessage::Shutdown(jsmsg::Dummy::default())).await;
          break;
        }
      }

      // Update terminal
      self.render()?;
    }

    self.task_tracker.wait().await;
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

  /// Shutdown TUI.
  pub fn shutdown_tui(&self) -> VoidIoResult {
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
}
