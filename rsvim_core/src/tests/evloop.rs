use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;
use crate::ui::canvas::Canvas;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jiff::Zoned;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::task::{Poll, Waker};
use std::time::Duration;

pub fn make_event_loop(terminal_cols: u16, terminal_rows: u16) -> EventLoop {
  let cli_opts = CliOptions::from_args(&vec!["--headless"]).unwrap();

  EventLoop::mock_new(terminal_cols, terminal_rows, cli_opts).unwrap()
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

const INTERVAL_MILLIS: Duration = Duration::from_millis(2);

#[derive(Debug)]
struct SharedWaker {
  pub waker: Option<Waker>,
}

#[derive(Debug)]
pub struct MockReader {
  rx: Receiver<IoResult<Event>>,
  shared_waker: Arc<Mutex<SharedWaker>>,
}

impl MockReader {
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

impl futures::Stream for MockReader {
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
