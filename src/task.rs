//! Async task.

use futures::Future;
use std::pin::Pin;

pub type TaskResult = Result<(), String>;
pub type Task = Pin<Box<dyn Future<Output = TaskResult>>>;
