//! The command-line search backward mode.

use crate::state::fsm::{StateDataAccess, StateMachine, Stateful};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CommandLineSearchBackwardStateful {}

impl Stateful for CommandLineSearchBackwardStateful {
  fn handle(&self, _data_access: StateDataAccess) -> StateMachine {
    StateMachine::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
}
