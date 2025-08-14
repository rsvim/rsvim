//! Event loop.

use crate::buf::{BuffersManager, BuffersManagerArc};
use crate::cli::CliOptions;
use crate::content::{TextContents, TextContentsArc};
use crate::js::msg::{
  self as jsmsg, EventLoopToJsRuntimeMessage, JsRuntimeToEventLoopMessage,
};
use crate::js::{JsRuntime, JsRuntimeOptions, SnapshotData};
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use msg::WorkerToMasterMessage;
use writer::{StdoutWritable, StdoutWriterValue};

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

pub mod msg;
pub mod task;
pub mod writer;

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

  /// Channel: "workers" => "master"
  ///
  /// Sender: workers send to master.
  pub worker_to_master: Sender<WorkerToMasterMessage>,
  /// Receiver: master receive from workers.
  pub master_from_worker: Receiver<WorkerToMasterMessage>,

  /// Channel: "master" => "js runtime"
  /// NOTE: In variables naming, we use "jsrt" for "js runtime".
  ///
  /// Receiver: master receive from js runtime.
  pub master_from_jsrt: Receiver<JsRuntimeToEventLoopMessage>,
  /// Sender: master send to js runtime.
  pub master_to_jsrt: Sender<EventLoopToJsRuntimeMessage>,

  /// Channel: "master" => "master" ("dispatcher" => "queue")
  ///
  /// Sender: dispatcher.
  pub jsrt_tick_dispatcher: Sender<EventLoopToJsRuntimeMessage>,
  /// Receiver: queue.
  pub jsrt_tick_queue: Receiver<EventLoopToJsRuntimeMessage>,
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
    cols: u16,
    rows: u16,
    cli_opts: &CliOptions,
  ) -> IoResult<(
    /* startup_moment */ Instant,
    /* startup_unix_epoch */ u128,
    /* canvas */ CanvasArc,
    /* tree */ TreeArc,
    /* state */ StateArc,
    /* stateful_machine */ StatefulValue,
    /* buffers */ BuffersManagerArc,
    /* contents */ TextContentsArc,
    /* writer */ StdoutWriterValue,
    /* cancellation_token */ CancellationToken,
    /* detached_tracker */ TaskTracker,
    /* blocked_tracker */ TaskTracker,
    /* worker_to_master */ Sender<WorkerToMasterMessage>,
    /* master_from_worker */ Receiver<WorkerToMasterMessage>,
    /* jsrt_to_master */ Sender<JsRuntimeToEventLoopMessage>,
    /* master_from_jsrt */ Receiver<JsRuntimeToEventLoopMessage>,
    /* master_to_jsrt */ Sender<EventLoopToJsRuntimeMessage>,
    /* jsrt_from_master */ Receiver<EventLoopToJsRuntimeMessage>,
    /* jsrt_tick_dispatcher */ Sender<EventLoopToJsRuntimeMessage>,
    /* jsrt_tick_queue */ Receiver<EventLoopToJsRuntimeMessage>,
  )> {
    // Canvas
    let canvas_size = U16Size::new(cols, rows);
    let canvas = Canvas::new(canvas_size);
    let canvas = Canvas::to_arc(canvas);

    // UI Tree
    let tree = Tree::to_arc(Tree::new(canvas_size));

    // Buffers
    let buffers_manager = BuffersManager::to_arc(BuffersManager::new());
    let text_contents = TextContents::to_arc(TextContents::new(canvas_size));

    // Channel: workers => master
    let (worker_to_master, master_from_worker) = channel(*CHANNEL_BUF_SIZE);

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
    // - Channel-1 `jsrt_to_master` => `master_from_jsrt`, on message `JsRuntimeToEventLoopMessage`.
    // - Channel-2 `jsrt_tick_dispatcher` => `jsrt_tick_queue`, on message `EventLoopToJsRuntimeMessage`.
    // - Channel-3 `master_to_jsrt` => `jsrt_from_master`, on message `EventLoopToJsRuntimeMessage`.
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
    let (jsrt_to_master, master_from_jsrt) = channel(*CHANNEL_BUF_SIZE);
    // Channel: master => js runtime
    let (master_to_jsrt, jsrt_from_master) = channel(*CHANNEL_BUF_SIZE);
    // Channel: master => master
    let (jsrt_tick_dispatcher, jsrt_tick_queue) = channel(*CHANNEL_BUF_SIZE);

    // Startup time
    let startup_moment = Instant::now();
    let startup_unix_epoch = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_millis();

    // State
    let state = State::to_arc(State::new(jsrt_tick_dispatcher.clone()));
    let stateful_machine = StatefulValue::default();

    let writer = if cli_opts.headless() {
      StdoutWriterValue::headless()
    } else {
      StdoutWriterValue::editor()
    };

    Ok((
      startup_moment,
      startup_unix_epoch,
      canvas,
      tree,
      state,
      stateful_machine,
      buffers_manager,
      text_contents,
      writer,
      CancellationToken::new(),
      TaskTracker::new(),
      TaskTracker::new(),
      worker_to_master,
      master_from_worker,
      jsrt_to_master,
      master_from_jsrt,
      master_to_jsrt,
      jsrt_from_master,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
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
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      worker_to_master,
      master_from_worker,
      jsrt_to_master,
      master_from_jsrt,
      master_to_jsrt,
      jsrt_from_master,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
    ) = Self::_internal_new(cols, rows, &cli_opts)?;

    // Js Runtime
    let js_runtime = JsRuntime::new(
      JsRuntimeOptions::default(),
      snapshot,
      startup_moment,
      startup_unix_epoch,
      jsrt_to_master,
      jsrt_from_master,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
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
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      js_runtime,
      worker_to_master,
      master_from_worker,
      master_from_jsrt,
      master_to_jsrt,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
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
      _writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      worker_to_master,
      master_from_worker,
      jsrt_to_master,
      master_from_jsrt,
      master_to_jsrt,
      jsrt_from_master,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
    ) = Self::_internal_new(terminal_columns, terminal_rows, &cli_opts)?;

    let writer = StdoutWriterValue::mock();

    // Js Runtime
    let js_runtime = JsRuntime::new_without_snapshot(
      JsRuntimeOptions::default(),
      startup_moment,
      startup_unix_epoch,
      jsrt_to_master,
      jsrt_from_master,
      cli_opts.clone(),
      tree.clone(),
      buffers.clone(),
      contents.clone(),
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
      writer,
      cancellation_token,
      detached_tracker,
      blocked_tracker,
      js_runtime,
      worker_to_master,
      master_from_worker,
      master_from_jsrt,
      master_to_jsrt,
      jsrt_tick_dispatcher,
      jsrt_tick_queue,
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
      self
        .js_runtime
        .execute_module(config_entry.to_str().unwrap(), None)
        .unwrap();
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
    let _cmdline_id = cmdline.id();

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

  async fn process_worker_notify(
    &mut self,
    message: Option<WorkerToMasterMessage>,
  ) {
    trace!("Received {:?} message from workers", message);
  }

  async fn process_js_runtime_request(
    &mut self,
    message: Option<JsRuntimeToEventLoopMessage>,
  ) {
    if let Some(message) = message {
      match message {
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

  async fn process_js_runtime_response(
    &mut self,
    message: Option<EventLoopToJsRuntimeMessage>,
  ) {
    if let Some(message) = message {
      trace!("Process resp msg:{:?}", message);
      let _ = self.master_to_jsrt.send(message).await;
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
        worker_msg = self.master_from_worker.recv() => {
          self.process_worker_notify(worker_msg).await;
        }
        // Receive notification from js runtime => master
        js_req = self.master_from_jsrt.recv() => {
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
        // Receive notification from workers => master
        worker_msg = self.master_from_worker.recv() => {
          self.process_worker_notify(worker_msg).await;
        }
        // Receive notification from js runtime => master
        js_req = self.master_from_jsrt.recv() => {
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

      // Flush logic UI to terminal, i.e. print UI to stdout
      lock!(self.tree).draw(self.canvas.clone());
      self.writer.write(&mut lock!(self.canvas))?;
    }

    Ok(())
  }
}
