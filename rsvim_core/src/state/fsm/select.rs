//! The select mode.

use crate::state::ops::Operation;
use crate::state::{StateDataAccess, StateMachine, Stateful};

use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The select editing mode.
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
    StateMachine::SelectMode(SelectStateful::default())
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::SelectMode(SelectStateful::default())
  }
}
