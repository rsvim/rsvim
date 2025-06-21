//! Vim buffer's default options.

use crate::buf::opt::*;

pub const TAB_STOP: u16 = 8;

pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;

#[cfg(target_os = "windows")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Dos;

#[cfg(not(target_os = "windows"))]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Unix;
