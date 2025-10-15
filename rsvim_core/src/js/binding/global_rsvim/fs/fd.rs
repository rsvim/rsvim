//! File descriptor utilities.

use crate::js;
use crate::js::JsFuture;
use crate::js::JsRuntime;
use crate::js::JsRuntimeState;
use crate::js::JsTimerId;
use crate::js::binding;
use crate::js::command::def::CommandDefinition;
use crate::js::converter::*;
use crate::js::converter::*;
use crate::js::pending;
use crate::prelude::*;
use crate::state::ops::cmdline_ops;
use compact_str::CompactString;
use compact_str::ToCompactString;
use ringbuf::traits::RingBuffer;
use std::fs;
use std::fs::File;
use std::rc::Rc;

#[cfg(not(target_family = "windows"))]
fn to_fd(file: File) -> usize {
  use std::os::fd::IntoRawFd;
  file.into_raw_fd() as usize
}

#[cfg(target_family = "windows")]
fn to_fd(file: File) -> usize {
  use std::os::windows::io::IntoRawHandle;
  file.into_raw_handle() as usize
}

#[cfg(not(target_family = "windows"))]
fn from_fd(fd: usize) -> File {
  use std::os::fd::FromRawFd;
  unsafe { File::from_raw_fd(fd as std::os::fd::RawFd) }
}

#[cfg(target_family = "windows")]
fn from_fd(handle: usize) -> File {
  use std::os::windows::io::FromRawHandle;
  unsafe { File::from_raw_handle(handle as std::os::windows::io::RawHandle) }
}
