//! The command-line mode.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line editing mode.
pub struct CommandLineExCommandVariantStateful {}

impl Stateful for CommandLineExCommandVariantStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineMode(CommandLineExCommandVariantStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::CommandLineMode(CommandLineExCommandVariantStateful::default())
  }
}
