//! Async task.

use futures::future::{BoxFuture, Future};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;
use tokio::task::{AbortHandle, JoinSet};

use crate::buf::BuffersArc;
use crate::evloop::message::Notify;
use crate::evloop::EventLoop;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod startup;

pub type TaskId = tokio::task::Id;
pub type TaskResult = Result<(), String>;
pub type TaskHandles = Arc<RwLock<HashMap<TaskId, AbortHandle>>>;

#[derive(Debug, Clone)]
/// The mutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
  pub worker_sender: UnboundedSender<Notify>,
}

impl TaskableDataAccess {
  pub fn new(
    state: StateArc,
    tree: TreeArc,
    buffers: BuffersArc,
    worker_sender: UnboundedSender<Notify>,
  ) -> Self {
    TaskableDataAccess {
      state,
      tree,
      buffers,
      worker_sender,
    }
  }
}
