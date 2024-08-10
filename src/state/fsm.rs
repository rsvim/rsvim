//! The finite-state machine for VIM's editing mode.
//! The editing mode of the editor is a global state, and moves from one to another.

use crate::state::mode::Mode;
use crate::ui::tree::TreeArc;

// Re-export
pub use crate::state::fsm::normal_stateful::NormalStateful;
pub use crate::state::fsm::select_stateful::SelectStateful;
pub use crate::state::fsm::visual_stateful::VisualStateful;

pub mod normal_stateful;
pub mod select_stateful;
pub mod visual_stateful;

#[derive(Debug, Clone)]
pub struct StatefulDataAccess {
  pub tree: TreeArc,
}

impl StatefulDataAccess {
  pub fn new(tree: TreeArc) -> Self {
    StatefulDataAccess { tree }
  }
}

pub trait Stateful {
  /// Handle user's keyboard/mouse event, this method can access the global state and update UI tree.
  ///
  /// Returns next state.
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful;

  /// Returns VIM mode.
  fn mode(&self) -> Mode;
}

#[derive(Debug, Copy, Clone)]
pub enum NextStateful {
  Normal(NormalStateful),
  Visual(VisualStateful),
  Select(SelectStateful),
}

impl Default for NextStateful {
  fn default() -> Self {
    NextStateful::Normal(NormalStateful::default())
  }
}

impl Stateful for NextStateful {
  fn handle(&self, data_access: StatefulDataAccess) -> NextStateful {
    match self {
      NextStateful::Normal(h) => h.handle(data_access),
      NextStateful::Visual(h) => h.handle(data_access),
      NextStateful::Select(h) => h.handle(data_access),
    }
  }

  fn mode(&self) -> Mode {
    match self {
      NextStateful::Normal(h) => h.mode(),
      NextStateful::Visual(h) => h.mode(),
      NextStateful::Select(h) => h.mode(),
    }
  }
}
