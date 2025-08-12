use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jiff::Zoned;
use std::sync::mpsc::{Receiver, channel};
use std::task::Poll;
use std::time::Duration;

pub fn make_event_loop() -> EventLoop {
  let cli_opts = CliOptions::from_args(&vec!["--headless"]).unwrap();

  EventLoop::new_without_snapshot(cli_opts).unwrap()
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
pub struct MockReader {
  rx: Receiver<IoResult<Event>>,
}

impl MockReader {
  pub fn new(events: Vec<MockEvent>) -> Self {
    let (tx, rx) = channel();

    std::thread::spawn(move || {
      for (i, event) in events.iter().enumerate() {
        trace!("Send mock event[{i}]: {event:?}");

        match event {
          MockEvent::Event(e) => {
            std::thread::sleep(INTERVAL_MILLIS);
            tx.send(Ok(e.clone()));
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
      }

      std::thread::sleep(INTERVAL_MILLIS);
      tx.send(Ok(CTRL_D.clone()));
    });

    Self { rx }
  }
}

impl futures::Stream for MockReader {
  type Item = IoResult<Event>;

  fn poll_next(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    match self.rx.try_recv() {
      Ok(event) => Poll::Ready(Some(event)),
      _ => Poll::Pending,
    }
  }
}
