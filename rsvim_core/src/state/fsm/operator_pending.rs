//! The operator-pending mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default)]
/// The operator-pending editing mode.
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::OperatorPendingMode(OperatorPendingStateful::default())
  }
}
