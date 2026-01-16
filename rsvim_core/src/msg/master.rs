//! Messages that are sent to [`EventLoop`](crate::evloop::EventLoop), here
//! call it "master".

use crate::js::TaskId;
use crate::js::TimerId;
use crate::js::binding::global_rsvim::fs::open::FsOpenOptions;
use std::path::PathBuf;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;

#[derive(Debug)]
/// Message sent to [`EventLoop`](crate::evloop::EventLoop).
pub enum MasterMessage {
  /// Js runtime ask master to set timeout, i.e. sleep
  TimeoutReq(TimeoutReq),

  /// Js runtime ask master to exit
  ExitReq(ExitReq),

  /// Js runtime ask master to load import
  LoadImportReq(LoadImportReq),

  /// Js runtime ask master to tick loop again.
  TickAgainReq,

  /// Js runtime ask master to open file.
  FsOpenReq(FsOpenReq),

  /// Js runtime ask master to read file.
  FsReadReq(FsReadReq),

  /// Js runtime ask master to write file.
  FsWriteReq(FsWriteReq),
}

#[derive(Debug)]
pub struct ExitReq {
  pub exit_code: i32,
}

#[derive(Debug)]
pub struct TimeoutReq {
  pub timer_id: TimerId,
  pub start_at: Instant,
  pub delay: u32,
  pub repeated: bool,
}

#[derive(Debug)]
pub struct LoadImportReq {
  pub task_id: TaskId,
  pub specifier: String,
}

#[derive(Debug)]
pub struct FsOpenReq {
  pub task_id: TaskId,
  pub path: PathBuf,
  pub options: FsOpenOptions,
}

#[derive(Debug)]
pub struct FsReadReq {
  pub task_id: TaskId,
  pub fd: usize,
  pub bufsize: usize,
}

#[derive(Debug)]
pub struct FsWriteReq {
  pub task_id: TaskId,
  pub fd: usize,
  pub buf: Vec<u8>,
}

/// Send master message in sync/blocking way, with tokio's "current_runtime".
pub fn send_to_master(
  master_tx: UnboundedSender<MasterMessage>,
  message: MasterMessage,
) {
  master_tx.send(message).unwrap();
}
