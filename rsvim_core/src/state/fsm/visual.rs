//! The visual mode.

use crate::state::ops::Operation;
use crate::state::{StateDataAccess, StateMachine, Stateful};

use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
    StateMachine::VisualMode(VisualStateful::default())
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::VisualMode(VisualStateful::default())
  }
}
