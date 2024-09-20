use futures::future::{BoxFuture, Future};
use ropey::{Rope, RopeBuilder};
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, error};

use crate::buf::{Buffer, BufferArc, Buffers, BuffersArc};
use crate::evloop::msg::{Dummy, WorkerToMasterMessage};
use crate::evloop::task::{TaskResult, TaskableDataAccess};
use crate::glovar;
use crate::result::ErrorCode;

/// Register a forever loop job to run in a fixed rate, each time this job simply send a message to
/// master.
///
/// Since the JsRuntime is completely running in another single thread, and all the operations
/// happened in the JsRuntime is unknown to EventLoop thread, unless it send channels. But when
/// implementing the operations for JsRuntime, if we always send a message in an operation, then
/// JsRuntime can send too many messages when it's evaluating the config file on start up, which we
/// simply cannot control.
///
/// By looping this job in a fixed rate, it may bring some extra CPU usage, but avoid a flood of
/// messages sent from JsRuntime to EventLoop.
pub async fn update_in_fixed_rate(data_access: TaskableDataAccess, millis: u64) -> TaskResult {
  let worker_send_to_master = data_access.worker_send_to_master;

  debug!("Start forever loop to update in fixed FPS rate");
  loop {
    worker_send_to_master
      .send(WorkerToMasterMessage::Dummy(Dummy::default()))
      .await
      .unwrap();
    tokio::time::sleep(Duration::from_millis(millis)).await;
  }
}
