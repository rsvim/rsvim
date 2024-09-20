//! Async task.

use futures::future::{BoxFuture, Future};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::task::{AbortHandle, JoinSet};
use tokio_util::task::TaskTracker;

use crate::buf::BuffersArc;
use crate::evloop::msg::WorkerToMasterMessage;
use crate::evloop::EventLoop;
use crate::result::VoidResult;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod startup;

pub type TaskResult = VoidResult;

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
