//! File descriptor/handle.

#[cfg(not(target_family = "windows"))]
pub fn as_raw_fd(file: &std::fs::File) -> usize {
  use std::os::fd::AsRawFd;
  file.as_raw_fd() as usize
}

#[cfg(target_family = "windows")]
pub fn as_raw_fd(file: &std::fs::File) -> usize {
  use std::os::windows::io::AsRawHandle;
  file.as_raw_handle() as usize
}
