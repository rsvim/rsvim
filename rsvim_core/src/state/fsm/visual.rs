//! The visual mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValueDispatcher};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValueDispatcher {
    StatefulValueDispatcher::VisualMode(VisualStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValueDispatcher {
    StatefulValueDispatcher::VisualMode(VisualStateful::default())
  }
}
