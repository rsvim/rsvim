//! The visual mode.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    StateMachine::VisualMode(VisualStateful::default())
  }
}
