//! Vim buffer's default options.
//!
//! See: [`crate::buf::BufferLocalOptions`].

use crate::buf::FileEncodingOption;
use crate::buf::FileFormatOption;

pub const TAB_STOP: u16 = 8;

pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;

#[cfg(target_os = "windows")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Dos;

#[cfg(target_os = "macos")]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Mac;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const FILE_FORMAT: FileFormatOption = FileFormatOption::Unix;
