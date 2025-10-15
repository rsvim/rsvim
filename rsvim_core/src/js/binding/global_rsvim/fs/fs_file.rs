//! File descriptor utilities.

use std::fs::File;

#[cfg(not(target_family = "windows"))]
pub fn to_fd(file: File) -> usize {
  use std::os::fd::IntoRawFd;
  file.into_raw_fd() as usize
}

#[cfg(target_family = "windows")]
pub fn to_fd(file: File) -> usize {
  use std::os::windows::io::IntoRawHandle;
  file.into_raw_handle() as usize
}

#[cfg(not(target_family = "windows"))]
pub fn from_fd(fd: usize) -> File {
  use std::os::fd::FromRawFd;
  unsafe { File::from_raw_fd(fd as std::os::fd::RawFd) }
}

#[cfg(target_family = "windows")]
pub fn from_fd(handle: usize) -> File {
  use std::os::windows::io::FromRawHandle;
  unsafe { File::from_raw_handle(handle as std::os::windows::io::RawHandle) }
}
