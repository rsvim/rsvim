//! Crossterm EventStream reader.

use crossterm::event::{Event, EventStream};
use futures::stream::Stream;

pub struct EventStreamReader {
    in: EventStream,
}

impl Stream for EventStreamReader {

}
