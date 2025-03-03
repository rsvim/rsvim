//! The command-line mode.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The command-line editing mode.
pub struct CommandLineStateful {}

impl Stateful for CommandLineStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    StateMachine::CommandLineMode(CommandLineStateful::default())
  }
}
