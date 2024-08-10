//! The normal mode.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct NormalStateful {}

impl Stateful for NormalStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::Normal(NormalStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Normal
  }
}
