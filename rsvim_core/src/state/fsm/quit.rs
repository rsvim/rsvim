//! The quit state.

use crate::state::ops::Operation;
use crate::state::{StateDataAccess, StateMachine, Stateful};

use crossterm::event::Event;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The quit state.
///
/// NOTE: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(
    &self,
    _data_access: StateDataAccess,
    _event: Event,
  ) -> StateMachine {
    // unreachable!("Never handle QuitStateful");
    StateMachine::QuitState(QuitStateful::default())
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    // unreachable!("Never handle QuitStateful");
    StateMachine::QuitState(QuitStateful::default())
  }
}
