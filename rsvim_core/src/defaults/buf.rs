//! Vim buffer's default options.
//!
//! See: [`crate::buf::BufferLocalOptions`].

use crate::buf::FileEncodingOption;

pub const TAB_STOP: u16 = 8;

pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;
