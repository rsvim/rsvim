//! Async task.

use futures::Future;
use std::pin::Pin;

use crate::buf::BuffersArc;
use crate::state::State;
use crate::ui::tree::TreeArc;

pub type TaskResult = Result<(), String>;
pub type Task = Pin<Box<dyn Future<Output = TaskResult>>>;

#[derive(Debug)]
/// The mutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccessMut<'a> {
  pub state: &'a mut State,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl<'a> TaskableDataAccessMut<'a> {
  pub fn new(state: &'a mut State, tree: TreeArc, buffers: BuffersArc) -> Self {
    TaskableDataAccessMut {
      state,
      tree,
      buffers,
    }
  }
}

#[derive(Debug, Clone)]
/// The immutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccess<'a> {
  pub state: &'a State,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl<'a> TaskableDataAccess<'a> {
  pub fn new(state: &'a State, tree: TreeArc, buffers: BuffersArc) -> Self {
    TaskableDataAccess {
      state,
      tree,
      buffers,
    }
  }
}
