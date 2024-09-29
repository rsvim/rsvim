//! The quit state.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default)]
/// The quit state.
///
/// NOTE: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    // unreachable!("Never handle QuitStateful");
    StatefulValue::QuitState(QuitStateful::default())
  }
}
