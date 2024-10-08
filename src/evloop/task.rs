//! Async task.

use tokio::sync::mpsc::Sender;

use crate::buf::BuffersArc;
use crate::evloop::msg::WorkerToMasterMessage;
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

pub mod startup;

#[derive(Debug, Clone)]
/// The mutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
  pub worker_send_to_master: Sender<WorkerToMasterMessage>,
}

impl TaskableDataAccess {
  pub fn new(
    state: StateArc,
    tree: TreeArc,
    buffers: BuffersArc,
    worker_send_to_master: Sender<WorkerToMasterMessage>,
  ) -> Self {
    TaskableDataAccess {
      state,
      tree,
      buffers,
      worker_send_to_master,
    }
  }
}
