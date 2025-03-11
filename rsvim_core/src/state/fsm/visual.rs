//! The visual mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The visual editing mode.
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::VisualMode(VisualStateful::default())
  }
}
