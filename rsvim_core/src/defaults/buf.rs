//! Vim buffer's options default value.

use crate::buf::opt::*;

pub const TAB_STOP: u16 = 8;
pub const EXPAND_TAB: bool = false;
pub const SHIFT_WIDTH: u16 = 8;
pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;

#[cfg(target_os = "windows")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Dos;

#[cfg(not(target_os = "windows"))]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Unix;
