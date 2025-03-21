//! Vim buffer's default options.

use crate::buf::FileEncodingOption;

/// Buffer 'tab-stop' option.
/// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
pub const TAB_STOP: u16 = 8;

/// Buffer 'file-encoding' option.
/// See: <https://vimhelp.org/options.txt.html#%27fileencoding%27>.
pub const FILE_ENCODING: FileEncodingOption = FileEncodingOption::Utf8;
