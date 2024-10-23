//! Vim window's default options.

/// Window 'wrap' option, also known as 'line-wrap', default to `true`.
/// See: <https://vimhelp.org/options.txt.html#%27wrap%27>.
pub const WRAP: bool = true;

/// Window 'line-break' option, also known as 'word-wrap', default to `false`.
/// See: <https://vimhelp.org/options.txt.html#%27linebreak%27>.
pub const LINE_BREAK: bool = false;

/// The 'break-at' option, default to `" ^I!@*-+;:,./?"`.
/// See: <https://vimhelp.org/options.txt.html#%27breakat%27>.
pub const BREAK_AT: &str = " ^I!@*-+;:,./?";

/// The 'tab-stop' option, default to `8`.
/// See: <https://vimhelp.org/options.txt.html#%27tabstop%27>.
pub const TAB_STOP: &str = " ^I!@*-+;:,./?";
