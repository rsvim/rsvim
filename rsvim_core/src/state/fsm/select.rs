//! The select mode.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The select editing mode.
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    StateMachine::SelectMode(SelectStateful::default())
  }
}
