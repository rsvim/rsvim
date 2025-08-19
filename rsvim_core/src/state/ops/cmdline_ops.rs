//! Command-line operations.

use crate::content::TextContents;
use crate::ui::tree::Inodeable;
use crate::ui::tree::Tree;
use crate::ui::viewport::Viewport;

use compact_str::CompactString;

pub fn set_cmdline_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: CompactString,
) {
  debug_assert!(tree.command_line().is_some());
  text_contents.command_line_message_mut().clear();
  text_contents
    .command_line_message_mut()
    .insert_at(0, 0, payload.clone());
  tree
    .command_line_mut()
    .unwrap()
    .update_message_viewport_by_text(text_contents.command_line_message());

  let opts = *tree.command_line().unwrap().options();
  let actual_shape = *tree.command_line().unwrap().message().actual_shape();

  let new_message_viewport = Viewport::to_arc(Viewport::view(
    &opts,
    text_contents.command_line_message(),
    &actual_shape,
    0,
    0,
  ));

  tree
    .command_line_mut()
    .unwrap()
    .set_message_viewport(new_message_viewport);
}
