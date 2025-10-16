//! File descriptor utilities.

pub const FD: &str = "fd";

#[cfg(not(target_family = "windows"))]
pub fn to_fd<F>(file: F) -> usize
where
  F: std::os::fd::AsRawFd,
{
  let fd = file.as_raw_fd();
  std::mem::forget(file);
  fd as usize
}

#[cfg(target_family = "windows")]
pub fn to_fd(file: F) -> usize
where
  F: std::os::windows::io::AsRawHandle,
{
  let handle = file.as_raw_handle();
  std::mem::forget(file);
  handle as usize
}

#[cfg(not(target_family = "windows"))]
pub fn from_fd<F>(fd: usize) -> F
where
  F: std::os::fd::FromRawFd,
{
  unsafe { F::from_raw_fd(fd as std::os::fd::RawFd) }
}

#[cfg(target_family = "windows")]
pub fn from_fd<F>(handle: usize) -> F
where
  F: std::os::windows::io::FromRawHandle,
{
  unsafe { F::from_raw_handle(handle as std::os::windows::io::RawHandle) }
}

#[derive(Debug)]
pub struct FsFile {}
