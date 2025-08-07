//! All STDOUT writers for rsvim.
//!
//! Rsvim has several running modes:
//! - Editor mode: The TUI text file editor.
//! - Headless mode (not implemented): The logical text editor without TUI. In
//!   this mode, the STDIN reads from command line instead of terminal's
//!   keyboard/mouse events, STDOUT/STDERR write to terminal instead of
//!   rendering TUI. Without TUI, the editing modes (normal, insert,
//!   command-line, visual, etc) is not useful any more, thus STDIN treats
//!   command line input as javascript scripts. And UI canvas no longer prints
//!   to STDOUT, instead, only message related APIs such as `console.log()`
//!   prints to STDOUT, which is similar to general purpose javascript-based
//!   runtime such as node/deno.

pub mod editor_mode_writer;
mod tui;
