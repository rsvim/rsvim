//! Async task/job.

#![allow(dead_code)]

use futures::Future;
use std::pin::Pin;

pub type TaskId = usize;
pub type TaskResult = Result<(), String>;
pub type Task = Pin<Box<dyn Future<Output = TaskResult>>>;
