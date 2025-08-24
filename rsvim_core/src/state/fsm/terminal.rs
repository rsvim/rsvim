//! The terminal mode.

use crate::state::fsm::{StateDataAccess, StateMachine, Stateful};
use crate::state::ops::Operation;

use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The terminal editing mode.
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
    StateMachine::TerminalMode(TerminalStateful::default())
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::TerminalMode(TerminalStateful::default())
  }
}
