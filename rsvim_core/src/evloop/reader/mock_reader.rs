//! Mocked event reader.

use crossterm::event::{
  Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use futures::stream::Stream;
use jiff::Zoned;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub enum MockEvent {
  Event(Event),

  /// The `CTRL-C` keyboard event, indicates exit the reader stream.
  ExitEvent,

  /// Sleep for a specific amount of time.
  SleepFor(Duration),

  /// Sleep until a specific time point.
  SleepUntil(Zoned),
}

#[derive(Debug)]
pub struct MockReader {}

impl MockReader {
  pub fn new() {}
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
