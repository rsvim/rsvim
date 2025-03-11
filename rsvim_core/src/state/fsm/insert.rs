//! The insert mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The insert editing mode.
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::InsertMode(InsertStateful::default())
  }
}
