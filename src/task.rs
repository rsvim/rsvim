//! Async task/job.

use futures::Future;
use std::io::Result as IoResult;

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
