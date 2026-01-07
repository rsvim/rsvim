//! The visual mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::VisualMode(VisualStateful::default())
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::VisualMode(VisualStateful::default())
  }
}
