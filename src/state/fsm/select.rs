//! The select mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::SelectMode(SelectStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Select
  }
}
