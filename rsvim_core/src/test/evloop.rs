use crate::cli::CliOptions;
use crate::evloop::EventLoop;
use crate::prelude::*;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::{State, StateArc};
use crate::ui::canvas::{Canvas, CanvasArc};
use crate::ui::tree::*;
use crate::ui::widget::command_line::CommandLine;
use crate::ui::widget::cursor::Cursor;
use crate::ui::widget::window::Window;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::stream::Stream;
use jiff::Zoned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use msg::WorkerToMasterMessage;
#[cfg(test)]
use reader::mock_reader::MockReader;
use writer::{StdoutWritable, StdoutWriterValue};

#[cfg(test)]
use bitflags::bitflags_match;
use crossterm::event::{Event, EventStream};
#[cfg(test)]
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use futures::stream::StreamExt;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub fn make_event_loop() -> EventLoop {
  let cli_opts = CliOptions::from_args(&vec!["--headless"]).unwrap();

  EventLoop::new_without_snapshot(cli_opts)
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

const INTERVAL_MILLIS: Duration = Duration::from_millis(50);

#[derive(Debug)]
pub struct MockReader {
  rx: UnboundedReceiver<IoResult<Event>>,
}

impl MockReader {
  pub fn new(events: Vec<MockEvent>) -> Self {
    let (tx, rx) = unbounded_channel::<IoResult<Event>>();

    thread::spawn(move || {
      for (i, event) in events.iter().enumerate() {
        trace!("Tick event[{i}]: {event:?}");
        match event {
          MockEvent::Event(e) => {
            thread::sleep(INTERVAL_MILLIS);
            tx.send(Ok(e.clone())).unwrap();
          }
          MockEvent::SleepFor(d) => thread::sleep(*d),
          MockEvent::SleepUntil(ts) => {
            let now = Zoned::now();
            let d = ts.duration_since(&now);
            let d = d.as_millis();
            if d > 0 {
              let d = Duration::from_millis(d as u64);
              thread::sleep(d)
            }
          }
        }
      }

      // No more events, send ExitEvent and close sender
      tx.send(Ok(CTRL_C.clone())).unwrap();
    });

    Self { rx }
  }
}

impl Stream for MockReader {
  type Item = IoResult<Event>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    self.rx.poll_recv(cx)
  }
}
