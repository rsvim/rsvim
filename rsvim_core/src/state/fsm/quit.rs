//! The quit state.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The quit state.
///
/// NOTE: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    // unreachable!("Never handle QuitStateful");
    StatefulValue::QuitState(QuitStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    // unreachable!("Never handle QuitStateful");
    StatefulValue::QuitState(QuitStateful::default())
  }
}
