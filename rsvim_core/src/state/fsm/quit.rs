//! The quit state.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValueDispatcher};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The quit state.
///
/// NOTE: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValueDispatcher {
    // unreachable!("Never handle QuitStateful");
    StatefulValueDispatcher::QuitState(QuitStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValueDispatcher {
    // unreachable!("Never handle QuitStateful");
    StatefulValueDispatcher::QuitState(QuitStateful::default())
  }
}
