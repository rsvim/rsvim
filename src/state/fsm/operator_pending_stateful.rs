//! The operator-pending mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::OperatorPending(OperatorPendingStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::OperatorPending
  }
}
