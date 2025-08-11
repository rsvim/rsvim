use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use jiff::Zoned;
use std::thread;
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

const CTRL_C: Event = Event::Key(KeyEvent::new_with_kind(
  KeyCode::Char('c'),
  KeyModifiers::CONTROL,
  KeyEventKind::Press,
));

const INTERVAL_MILLIS: Duration = Duration::from_millis(20);

#[derive(Debug)]
pub struct MockReader {
  events: Vec<MockEvent>,
  idx: usize,
}

impl MockReader {
  pub fn new(events: Vec<MockEvent>) -> Self {
    Self { events, idx: 0 }
  }

  pub async fn read(&mut self) -> Option<IoResult<Event>> {
    if self.idx >= self.events.len() {
      thread::sleep(INTERVAL_MILLIS);
      Some(Ok(CTRL_C.clone()))
    } else {
      let i = self.idx;
      let next_event = self.events[i];
      self.idx += 1;

      trace!("Tick event[{i}]: {next_event:?}");
      match next_event {
        MockEvent::Event(e) => {
          thread::sleep(INTERVAL_MILLIS);
          Some(Ok(e.clone()))
        }
        MockEvent::SleepFor(d) => {
          tokio::time::sleep(d).await;
          None
        }
        MockEvent::SleepUntil(ts) => {
          let now = Zoned::now();
          let d = ts.duration_since(&now);
          let d = d.as_millis();
          if d > 0 {
            let d = Duration::from_millis(d as u64);
            tokio::time::sleep(d).await;
          }
          None
        }
      }
    }
  }
}
