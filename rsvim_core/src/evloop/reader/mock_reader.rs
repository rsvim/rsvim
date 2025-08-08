//! Mocked event reader.

use crate::prelude::*;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::stream::Stream;
use jiff::{SignedDuration, Span, ToSpan, Zoned};
use std::pin::Pin;
use std::sync::mpsc::channel;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockEvent {
  Event(Event),

  /// The `CTRL-C` keyboard event, indicates exit the reader stream.
  ExitEvent,

  /// Sleep for a specific amount of time.
  SleepFor(Duration),

  /// Sleep until a specific time point.
  SleepUntil(Zoned),
}

const CTRL_C_EVENT: Event = Event::Key(KeyEvent::new_with_kind(
  KeyCode::Char('c'),
  KeyModifiers::CONTROL,
  KeyEventKind::Press,
));

#[derive(Debug)]
pub struct MockReader {}

impl MockReader {
  pub fn new(events: Vec<MockEvent>) -> Self {
    let (tx, rx) = channel::<Event>();

    thread::spawn(move || {
      for (i, event) in events.iter().enumerate() {
        match event {
          MockEvent::Event(e) => tx.send(e.clone()).unwrap(),
          MockEvent::ExitEvent => tx.send(CTRL_C_EVENT.clone()).unwrap(),
          MockEvent::SleepFor(d) => thread::sleep(*d),
          MockEvent::SleepUntil(ts) => {
            let now = Zoned::now();
            let diff = ts.duration_since(&now);
            let diff = diff.as_millis();
            if diff > 0 {
              let diff = Duration::from_millis(diff as u64);
              thread::sleep(diff)
            }
          }
        }
      }
      Ok(())
    });
  }
}

impl Stream for MockReader {
  type Item = std::io::Result<Event>;

  fn poll_next(
    self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    Poll::Ready(Some(Ok(Event::Key(key_event))))
  }
}
