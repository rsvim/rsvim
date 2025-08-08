//! Mocked event reader.

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct MockReader {}

impl Stream for MockReader {
  type Item = std::io::Result<Event>;

  fn poll_next(
    self: Pin<&mut Self>,
    _ctx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let key_event = KeyEvent::new_with_kind(
      KeyCode::Char('a'),
      KeyModifiers::empty(),
      KeyEventKind::Press,
    );
    Poll::Ready(Some(Ok(Event::Key(key_event))))
  }
}
