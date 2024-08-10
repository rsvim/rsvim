//! The quit state.

use crossterm::event::{
  DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture, Event,
  EventStream, KeyCode, KeyEventKind, KeyEventState, KeyModifiers,
};
use std::time::Duration;

use crate::glovar;
use crate::state::fsm::{Stateful, StatefulDataAccess, StatefulValue};
use crate::state::mode::Mode;

#[derive(Debug, Copy, Clone, Default)]
/// The quit
pub struct QuitStateful {}

impl Stateful for QuitStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> StatefulValue {
    StatefulValue::QuitState(QuitStateful::default())
  }

  fn mode(&self) -> Mode {}
}
