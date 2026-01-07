//! The select mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The select editing mode.
pub struct Select {}

impl Stateful for Select {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::Select(Select::default())
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::Select(Select::default())
  }
}
