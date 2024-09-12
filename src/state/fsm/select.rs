//! The select mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The select editing mode.
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    data_access.state.set_mode(Mode::Select);
    StatefulValue::SelectMode(SelectStateful::default())
  }
}
