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

pub mod tokio {
  use tokio::fs::File;

  #[cfg(not(target_family = "windows"))]
  pub fn to_fd(file: File) -> usize {
    use std::os::fd::AsRawFd;
    file.as_raw_fd() as usize
  }

  #[cfg(target_family = "windows")]
  pub fn to_fd(file: File) -> usize {
    use tokio::os::windows::io::IntoRawHandle;
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
}

#[derive(Debug)]
pub struct FsFile {}
