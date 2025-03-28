//! Vim window's default options.

/// Window 'wrap' option, also known as 'line-wrap', default to `true`.
/// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
pub const WRAP: bool = true;

/// Window 'line-break' option, also known as 'word-wrap', default to `false`.
/// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
pub const LINE_BREAK: bool = false;

/// Window 'scroll-off' option, default to `0`.
/// See: <https://vimhelp.org/options.txt.html#%27scrolloff%27>.
pub const SCROLL_OFF: u16 = 0_u16;
