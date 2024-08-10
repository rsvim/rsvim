//! The insert mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::InsertMode(InsertStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Insert
  }
}
