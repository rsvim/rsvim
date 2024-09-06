//! Async task.

use futures::future::{BoxFuture, Future};
use std::pin::Pin;
use std::sync::Arc;
use tokio::task::JoinHandle;

use crate::buf::BuffersArc;
use crate::evloop::EventLoop;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod startup;

pub type TaskId = tokio::task::Id;
pub type TaskResult = Result<(), String>;

#[derive(Debug)]
/// The mutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccess {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl TaskableDataAccess {
  pub fn new(state: StateArc, tree: TreeArc, buffers: BuffersArc) -> Self {
    TaskableDataAccess {
      state,
      tree,
      buffers,
    }
  }
}
