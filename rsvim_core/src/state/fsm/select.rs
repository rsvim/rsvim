//! The select mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The select editing mode.
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::SelectMode(SelectStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::SelectMode(SelectStateful::default())
  }
}
