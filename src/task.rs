//! Async task/job.

#![allow(dead_code)]

use futures::Future;
use std::io::Result as IoResult;
use std::sync::Arc;

use crate::cli::CliOpt;
use crate::evloop::EventLoop;
use crate::state::StateArc;
use crate::ui::tree::TreeArc;

pub type TaskId = usize;

pub mod startup;

pub trait Task: Future {
  fn id(&self) -> TaskId;
}

pub enum TaskValue {}

impl Future for TaskValue {
  type Output = IoResult<bool>;

  fn poll(
    self: std::pin::Pin<&mut Self>,
    _cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    match *self {}
  }
}

#[derive(Debug, Clone)]
/// Shareable context passed through async threads.
///
/// Note: This is almost a copy of [`EventLoop`] that contains all global editor data structures.
pub struct TaskContext {
  cli_opt: CliOpt,
  tree: TreeArc,
  state: StateArc,
}

impl From<EventLoop> for TaskContext {
  fn from(value: EventLoop) -> Self {
    TaskContext {
      cli_opt: value.cli_opt.clone(),
      tree: Arc::clone(&value.tree),
      state: Arc::clone(&value.state),
    }
  }
}
