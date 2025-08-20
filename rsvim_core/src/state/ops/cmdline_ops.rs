//! Command-line operations.

use crate::content::TextContents;
use crate::ui::tree::{Inodeable, Tree};
use crate::ui::viewport::Viewport;

use compact_str::CompactString;

pub fn cmdline_set_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: CompactString,
) {
  debug_assert!(tree.command_line().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();
  message_text.insert_at(0, 0, payload);

  let cmdline = tree.command_line_mut().unwrap();
  let opts = *cmdline.options();
  let actual_shape = *cmdline.message().actual_shape();

  let new_message_viewport =
    Viewport::to_arc(Viewport::view(&opts, message_text, &actual_shape, 0, 0));

  cmdline.set_message_viewport(new_message_viewport);
}

pub fn cmdline_clear_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
) {
  debug_assert!(tree.command_line().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();

  let cmdline = tree.command_line_mut().unwrap();
  let opts = *cmdline.options();
  let actual_shape = *cmdline.message().actual_shape();

  let new_message_viewport =
    Viewport::to_arc(Viewport::view(&opts, message_text, &actual_shape, 0, 0));

  cmdline.set_message_viewport(new_message_viewport);
}
