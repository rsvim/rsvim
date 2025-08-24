//! The operator-pending mode.

use crate::state::fsm::{StateDataAccess, StateMachine, Stateful};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The operator-pending editing mode.
pub struct OperatorPendingStateful {}

impl Stateful for OperatorPendingStateful {
  fn handle(&self, _data_access: StateDataAccess) -> StateMachine {
    StateMachine::OperatorPendingMode(OperatorPendingStateful::default())
  }
  fn handle_op(
    &self,
    _data_access: StateDataAccess,
    _op: Operation,
  ) -> StateMachine {
    StateMachine::OperatorPendingMode(OperatorPendingStateful::default())
  }
}
