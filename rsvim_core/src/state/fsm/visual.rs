//! The visual mode.

use crate::state::State;
use crate::state::StateContext;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The visual editing mode.
pub struct Visual {}

impl Stateful for Visual {
  fn handle(&self, _context: &StateContext, _event: Event) -> State {
    State::Visual(Visual::default())
  }
  fn handle_op(&self, _context: &StateContext, _op: Operation) -> State {
    State::Visual(Visual::default())
  }
}
