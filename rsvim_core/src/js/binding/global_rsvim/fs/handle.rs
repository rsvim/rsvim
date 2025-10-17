//! File handle.

pub const FD: &str = "fd";

#[cfg(not(target_family = "windows"))]
pub fn to_fd<F>(file: F) -> usize
where
  F: std::os::fd::IntoRawFd,
{
  file.into_raw_fd() as usize
}

#[cfg(target_family = "windows")]
pub fn to_fd<F>(file: F) -> usize
where
  F: std::os::windows::io::IntoRawHandle,
{
  file.as_raw_handle() as usize
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
