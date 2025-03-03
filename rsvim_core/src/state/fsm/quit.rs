//! The quit state.

use crate::state::fsm::{StateMachine, Stateful, StatefulDataAccess};

#[derive(Debug, Copy, Clone, Default)]
/// The quit state.
///
/// NOTE: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StateMachine {
    // unreachable!("Never handle QuitStateful");
    StateMachine::QuitState(QuitStateful::default())
  }
}
