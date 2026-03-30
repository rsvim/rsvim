//! The terminal mode.

use crate::state::State;
use crate::state::StateContext;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The terminal editing mode.
pub struct Terminal {}

impl Stateful for Terminal {
  fn handle(&self, _context: &StateContext, _event: Event) -> State {
    State::Terminal(Terminal::default())
  }
  fn handle_op(&self, _context: &StateContext, _op: Operation) -> State {
    State::Terminal(Terminal::default())
  }
}
