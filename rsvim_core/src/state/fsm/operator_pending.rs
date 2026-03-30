//! The operator-pending mode.

use crate::state::State;
use crate::state::StateContext;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The operator-pending editing mode.
pub struct OperatorPending {}

impl Stateful for OperatorPending {
  fn handle(&self, _data_access: StateContext, _event: Event) -> State {
    State::OperatorPending(OperatorPending::default())
  }
  fn handle_op(&self, _data_access: StateContext, _op: Operation) -> State {
    State::OperatorPending(OperatorPending::default())
  }
}
