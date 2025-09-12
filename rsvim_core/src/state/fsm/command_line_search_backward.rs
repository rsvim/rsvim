//! The command-line search backward mode.

use crate::state::StateDataAccess;
use crate::state::StateMachine;
use crate::state::Stateful;
use crate::state::ops::Operation;
use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CommandLineSearchBackwardStateful {}

impl Stateful for CommandLineSearchBackwardStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
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
