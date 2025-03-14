//! The select mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The select editing mode.
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::SelectMode(SelectStateful::default())
  }
}
