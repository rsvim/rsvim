use crate::cfg::path_cfg::PathConfig;
use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;
use crate::state::ops::Operation;
use assert_fs::prelude::*;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use jiff::Zoned;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::task::Poll;
use std::task::Waker;
use std::thread_local;
use std::time::Duration;

#[derive(Debug)]
pub struct TempPathConfig {
  pub home_dir: assert_fs::TempDir,
  pub xdg_config_home: assert_fs::TempDir,
  pub xdg_cache_home: assert_fs::TempDir,
  pub xdg_data_home: assert_fs::TempDir,
}

thread_local! {
  pub static TEMP_PATH_CONFIG: RefCell<Option<PathConfig>> = RefCell::new(None);
}

impl TempPathConfig {
  pub fn create() -> Self {
    let temp_dirs = TempPathConfig {
      home_dir: assert_fs::TempDir::new().unwrap(),
      xdg_config_home: assert_fs::TempDir::new().unwrap(),
      xdg_cache_home: assert_fs::TempDir::new().unwrap(),
      xdg_data_home: assert_fs::TempDir::new().unwrap(),
    };
    TEMP_PATH_CONFIG.set(Some(PathConfig::_new_with_temp_dirs(&temp_dirs)));
    temp_dirs
  }
}

pub fn make_configs(sources: Vec<(&Path, &str)>) -> TempPathConfig {
  let tp = TempPathConfig::create();

  for (path, src) in sources.iter() {
    let path = tp.xdg_config_home.child("rsvim").child(path);
    path.touch().unwrap();
    std::fs::write(path, src).unwrap();
  }

  tp
}

pub fn make_home_configs(
  sources: Vec<(&Path, &str)>,
) -> (TempPathConfig, PathConfig) {
  let tp = TempPathConfig::create();

  for (path, src) in sources.iter() {
    let path = tp.home_dir.child(path);
    path.touch().unwrap();
    std::fs::write(path, src).unwrap();
  }

  let path_cfg = PathConfig::new_with_temp_dirs(&tp);
  (tp, path_cfg)
}

pub fn make_event_loop(
  terminal_cols: u16,
  terminal_rows: u16,
  cli_opts: CliOptions,
  path_cfg: PathConfig,
) -> EventLoop {
  EventLoop::mock_new(terminal_cols, terminal_rows, cli_opts, path_cfg).unwrap()
}

const INTERVAL_MILLIS: Duration = Duration::from_millis(2);

#[derive(Debug)]
struct SharedWaker {
  pub waker: Option<Waker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockEvent {
  /// Normal keyboard event
  Event(Event),

  /// Sleep for a specific amount of time.
  SleepFor(Duration),

  /// Sleep until a specific time point.
  SleepUntil(Zoned),
}

const CTRL_D: Event = Event::Key(KeyEvent::new_with_kind(
  KeyCode::Char('d'),
  KeyModifiers::CONTROL,
  KeyEventKind::Press,
));

#[derive(Debug)]
pub struct MockEventReader {
  rx: Receiver<IoResult<Event>>,
  shared_waker: Arc<Mutex<SharedWaker>>,
}

impl MockEventReader {
  pub fn new(events: Vec<MockEvent>) -> Self {
    let (tx, rx) = channel();
    let shared_waker = Arc::new(Mutex::new(SharedWaker { waker: None }));
    let cloned_shared_waker = shared_waker.clone();

    std::thread::spawn(move || {
      for (i, event) in events.iter().enumerate() {
        trace!("Send mock event[{i}]: {event:?}");

        match event {
          MockEvent::Event(e) => {
            std::thread::sleep(INTERVAL_MILLIS);
            tx.send(Ok(e.clone())).unwrap();
          }
          MockEvent::SleepFor(d) => {
            std::thread::sleep(*d);
          }
          MockEvent::SleepUntil(ts) => {
            let now = Zoned::now();
            let d = ts.duration_since(&now);
            let d = d.as_millis();
            if d > 0 {
              let d = Duration::from_millis(d as u64);
              std::thread::sleep(d);
            }
          }
        }

        let mut thread_shared_waker = cloned_shared_waker.lock();
        if let Some(waker) = thread_shared_waker.waker.take() {
          waker.wake();
        }
      }

      trace!("Send final mock event[{}]: CTRL+D {CTRL_D:?}", events.len());
      std::thread::sleep(INTERVAL_MILLIS);
      tx.send(Ok(CTRL_D.clone())).unwrap();

      let mut thread_shared_waker = cloned_shared_waker.lock();
      if let Some(waker) = thread_shared_waker.waker.take() {
        waker.wake();
      }
    });

    Self { rx, shared_waker }
  }
}

impl futures::Stream for MockEventReader {
  type Item = IoResult<Event>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    {
      let mut shared_waker = self.shared_waker.lock();
      shared_waker.waker = Some(cx.waker().clone());
    }
    match self.rx.try_recv() {
      Ok(event) => Poll::Ready(Some(event)),
      _ => Poll::Pending,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockOperation {
  /// Editor operation
  Operation(Operation),

  /// Sleep for a specific amount of time.
  SleepFor(Duration),

  /// Sleep until a specific time point.
  SleepUntil(Zoned),

  Exit,
}

const EXIT: MockOperation = MockOperation::Exit;

#[derive(Debug)]
pub struct MockOperationReader {
  rx: Receiver<IoResult<MockOperation>>,
  shared_waker: Arc<Mutex<SharedWaker>>,
}

impl MockOperationReader {
  pub fn new(operations: Vec<MockOperation>) -> Self {
    let (tx, rx) = channel();
    let shared_waker = Arc::new(Mutex::new(SharedWaker { waker: None }));
    let cloned_shared_waker = shared_waker.clone();

    std::thread::spawn(move || {
      for (i, op) in operations.iter().enumerate() {
        trace!("Send mock operation[{i}]: {op:?}");

        match op {
          MockOperation::Operation(op) => {
            std::thread::sleep(INTERVAL_MILLIS);
            tx.send(Ok(MockOperation::Operation(op.clone()))).unwrap();
          }
          MockOperation::SleepFor(d) => {
            std::thread::sleep(*d);
          }
          MockOperation::SleepUntil(ts) => {
            let now = Zoned::now();
            let d = ts.duration_since(&now);
            let d = d.as_millis();
            if d > 0 {
              let d = Duration::from_millis(d as u64);
              std::thread::sleep(d);
            }
          }
          MockOperation::Exit => {
            std::thread::sleep(INTERVAL_MILLIS);
            tx.send(Ok(MockOperation::Exit)).unwrap();
          }
        }

        let mut thread_shared_waker = cloned_shared_waker.lock();
        if let Some(waker) = thread_shared_waker.waker.take() {
          waker.wake();
        }
      }

      trace!(
        "Send final mock operation[{}]: Exit {:?}",
        operations.len(),
        EXIT
      );
      std::thread::sleep(INTERVAL_MILLIS);
      tx.send(Ok(EXIT.clone())).unwrap();

      let mut thread_shared_waker = cloned_shared_waker.lock();
      if let Some(waker) = thread_shared_waker.waker.take() {
        waker.wake();
      }
    });

    Self { rx, shared_waker }
  }
}

impl futures::Stream for MockOperationReader {
  type Item = IoResult<MockOperation>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    {
      let mut shared_waker = self.shared_waker.lock();
      shared_waker.waker = Some(cx.waker().clone());
    }
    match self.rx.try_recv() {
      Ok(op) => Poll::Ready(Some(op)),
      _ => Poll::Pending,
    }
  }
}
