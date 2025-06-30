//! The command-line search backward mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValueDispatcher};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line search backward mode.
pub struct CommandLineSearchBackwardStateful {}

impl Stateful for CommandLineSearchBackwardStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValueDispatcher {
    StatefulValueDispatcher::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValueDispatcher {
    StatefulValueDispatcher::CommandLineSearchBackwardMode(
      CommandLineSearchBackwardStateful::default(),
    )
  }
}
