//! The insert mode.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The insert editing mode.
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    StateMachine::InsertMode(InsertStateful::default())
  }
}
