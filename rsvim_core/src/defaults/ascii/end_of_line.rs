//! End of line.
//!
//! NOTE: The `CR` ("\r") is not implemented since ropey doesn't recognize it as a line break, and
//! it is quite legacy we don't use it.

pub const CRLF: &str = "\r\n";
pub const LF: &str = "\n";

// pub const CR: &str = "\r";
