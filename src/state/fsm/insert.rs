//! The insert mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct InsertStateful {}

impl Stateful for InsertStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::Insert(InsertStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Insert
  }
}
