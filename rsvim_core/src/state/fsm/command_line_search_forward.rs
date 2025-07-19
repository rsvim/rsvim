//! The command-line search forward mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search forward mode.
pub struct CommandLineSearchForwardStateful {}

impl Stateful for CommandLineSearchForwardStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
  fn handle_op(
    &self,
    _data_access: StatefulDataAccess,
    _op: Operation,
  ) -> StatefulValue {
    StatefulValue::CommandLineSearchForwardMode(
      CommandLineSearchForwardStateful::default(),
    )
  }
}
