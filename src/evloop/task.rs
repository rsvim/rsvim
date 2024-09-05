//! Async task.

use futures::future::{BoxFuture, Future};
use std::pin::Pin;

use crate::buf::BuffersArc;
use crate::evloop::EventLoop;
use crate::state::{State, StateArc};
use crate::ui::tree::TreeArc;

pub mod startup;

pub type TaskId = usize;
pub type TaskResult = Result<(), String>;
pub type TaskFuture = dyn Future<Output = TaskResult>;

pub trait Taskable: Future<Output = TaskResult> {
  fn id(&self) -> TaskId;
}

pub type Task = BoxFuture<'static, TaskResult>;

#[derive(Debug)]
/// The mutable data passed to task, and allow them access the editor.
pub struct TaskableDataAccessMut {
  pub state: StateArc,
  pub tree: TreeArc,
  pub buffers: BuffersArc,
}

impl TaskableDataAccessMut {
  pub fn new(state: StateArc, tree: TreeArc, buffers: BuffersArc) -> Self {
    TaskableDataAccessMut {
      state,
      tree,
      buffers,
    }
  }
}

// #[derive(Debug, Clone)]
// /// The immutable data passed to task, and allow them access the editor.
// pub struct TaskableDataAccess<'a> {
//   pub state: &'a State,
//   pub tree: TreeArc,
//   pub buffers: BuffersArc,
// }
//
// impl<'a> TaskableDataAccess<'a> {
//   pub fn new(state: &'a State, tree: TreeArc, buffers: BuffersArc) -> Self {
//     TaskableDataAccess {
//       state,
//       tree,
//       buffers,
//     }
//   }
// }
