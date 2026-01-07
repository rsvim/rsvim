//! The operator-pending mode.

use crate::state::State;
use crate::state::StateDataAccess;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The operator-pending editing mode.
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, _data_access: StateDataAccess, _event: Event) -> State {
    State::OperatorPendingMode(OperatorPendingStateful::default())
  }
  fn handle_op(&self, _data_access: StateDataAccess, _op: Operation) -> State {
    State::OperatorPendingMode(OperatorPendingStateful::default())
  }
}
