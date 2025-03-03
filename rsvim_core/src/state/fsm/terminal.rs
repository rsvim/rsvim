//! The terminal mode.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The terminal editing mode.
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    StateMachine::TerminalMode(TerminalStateful::default())
  }
}
