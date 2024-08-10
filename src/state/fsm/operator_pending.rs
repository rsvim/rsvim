//! The operator-pending mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::OperatorPendingMode(OperatorPendingStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::OperatorPending
  }
}
