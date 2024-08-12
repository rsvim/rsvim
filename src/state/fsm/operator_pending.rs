//! The operator-pending mode.

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The operator-pending editing mode.
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::OperatorPending);
    StatefulValue::OperatorPendingMode(OperatorPendingStateful::default())
  }
}
