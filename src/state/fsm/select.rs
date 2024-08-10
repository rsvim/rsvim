//! The select mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulDataAccessMut, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, data_access: StatefulDataAccessMut) -> StatefulValue {
    data_access.state.set_mode(Mode::Select);
    StatefulValue::SelectMode(SelectStateful::default())
  }
}
