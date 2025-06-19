//! The command-line mode, ex-command variant.

use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::ops::Operation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
/// The command-line mode, ex-command variant.
pub struct CommandLineExVariantStateful {}

impl Stateful for CommandLineExVariantStateful {
  fn handle(&self, _data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::CommandLineExMode(CommandLineExVariantStateful::default())
  }
  fn handle_op(&self, _data_access: StatefulDataAccess, _op: Operation) -> StatefulValue {
    StatefulValue::CommandLineExMode(CommandLineExVariantStateful::default())
  }
}
