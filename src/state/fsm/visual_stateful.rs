//! The visual mode editing state.

use crate::state::fsm::{NextStateful, Stateful, StatefulDataAccess};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
pub struct VisualStateful {}

impl Stateful for VisualStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful {
    NextStateful::Visual(VisualStateful::default())
  }

  fn mode(&self) -> Mode {
    Mode::Visual
  }
}
