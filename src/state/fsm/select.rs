//! The select mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct SelectStateful {}

impl Stateful for SelectStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::Select(SelectStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Select
  }
}
