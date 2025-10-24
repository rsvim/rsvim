//! File handle.

pub const FD: &str = "fd";

#[cfg(not(target_family = "windows"))]
pub fn std_to_fd(file: std::fs::File) -> usize {
  use std::os::fd::IntoRawFd;
  file.into_raw_fd() as usize
}

#[cfg(target_family = "windows")]
pub fn std_to_fd(file: std::fs::File) -> usize {
  use std::os::windows::io::IntoRawHandle;
  file.into_raw_handle() as usize
}

#[cfg(not(target_family = "windows"))]
pub unsafe fn std_from_fd(fd: usize) -> std::fs::File {
  use std::os::fd::FromRawFd;
  unsafe { std::fs::File::from_raw_fd(fd as std::os::fd::RawFd) }
}

#[cfg(target_family = "windows")]
pub unsafe fn std_from_fd(handle: usize) -> std::fs::File
where
{
  use std::os::windows::io::FromRawHandle;
  unsafe {
    std::fs::File::from_raw_handle(handle as std::os::windows::io::RawHandle)
  }
}

pub async fn tokio_to_fd(file: tokio::fs::File) -> usize {
  let file = file.into_std().await;
  std_to_fd(file)
}

pub fn tokio_from_fd(fd: usize) -> tokio::fs::File {
  let file = std_from_fd(fd);
  tokio::fs::File::from_std(file)
}
