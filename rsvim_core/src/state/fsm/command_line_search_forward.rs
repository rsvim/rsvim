//! The command-line search forward mode.

use crate::state::fsm::{StateDataAccess, StateMachine, Stateful};
use crate::state::ops::Operation;

use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search forward mode.
pub struct CommandLineSearchForwardStateful {}

impl Stateful for CommandLineSearchForwardStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
    StateMachine::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
}
