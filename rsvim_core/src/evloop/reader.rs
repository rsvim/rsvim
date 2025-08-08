//! STDIN readers for rsvim.

use crate::prelude::*;
use crate::ui::canvas::Canvas;

use crossterm::event::Event;
use futures::stream::Stream;

pub mod event_stream_reader;
pub mod mock_reader;

/// Async read from STDIN.
pub trait StdinReadable: Stream {}
