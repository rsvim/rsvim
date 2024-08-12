//! The quit state.

use crate::state::fsm::{Stateful, StatefulDataAccessMut, StatefulValue};

#[derive(Debug, Copy, Clone, Default)]
/// The quit state.
///
/// Note: This is an internal state to tell the editor to quit.
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, _data_access: StatefulDataAccessMut) -> StatefulValue {
    unreachable!("Never handle QuitStateful");
  }
}
