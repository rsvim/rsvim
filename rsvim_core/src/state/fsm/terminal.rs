//! The terminal mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValueDispatcher};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The terminal editing mode.
pub struct TerminalStateful {}

impl Stateful for TerminalStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValueDispatcher {
    StatefulValueDispatcher::TerminalMode(TerminalStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValueDispatcher {
    StatefulValueDispatcher::TerminalMode(TerminalStateful::default())
  }
}
