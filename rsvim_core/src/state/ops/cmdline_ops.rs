//! Command-line operations.

use crate::content::TextContents;
use crate::prelude::*;
use crate::ui::tree::Inodeable;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeNode;
use crate::ui::viewport::CursorViewport;
use crate::ui::viewport::Viewport;
use compact_str::ToCompactString;
use ringbuf::traits::Consumer;
use ringbuf::traits::RingBuffer;

fn set_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: String,
) {
  debug_assert!(tree.cmdline_id().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();
  message_text.insert_at(0, 0, payload.to_compact_string());

  let (cmdline_opts, cmdline_message_id) = {
    let cmdline = tree.cmdline();
    (*cmdline.options(), cmdline.message_id())
  };
  let actual_size = match tree.node(cmdline_message_id).unwrap() {
    TreeNode::CmdlineMessage(message) => message.actual_shape().size(),
    _ => unreachable!(),
  };

  let new_message_viewport =
    Viewport::view(&cmdline_opts, message_text, &actual_size, 0, 0);

  tree
    .cmdline_mut()
    .set_message_viewport(new_message_viewport);
}

pub fn cmdline_flush_pending_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
) {
  // If message history contains some payload. This means before we actually
  // running the event loop, there's already some messages wait for print.
  let maybe_last_msg =
    text_contents.command_line_message_history().last().cloned();
  trace!(
    "|cmdline_flush_pending_message| last_msg:{:?}",
    maybe_last_msg
  );
  if let Some(last_msg) = maybe_last_msg {
    // Current "command-line-message" widget can only print 1 single-line
    // message, multi-line messages are not support yet.
    //
    // FIXME: Fix me once our "command-line-message" widget support
    // multi-line messages.
    set_message(tree, text_contents, last_msg);
  }
}

pub fn cmdline_set_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: String,
) {
  set_message(tree, text_contents, payload.clone());

  // Also append message history:
  let cmdline_hist = text_contents.command_line_message_history_mut();
  cmdline_hist.push_overwrite(payload);
}

pub fn cmdline_clear_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
) {
  debug_assert!(tree.cmdline_id().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();

  let cmdline = tree.cmdline_mut().unwrap();
  let opts = *cmdline.options();
  let actual_size = cmdline.message().actual_shape().size();

  let new_message_viewport =
    Viewport::to_arc(Viewport::view(&opts, message_text, &actual_size, 0, 0));

  cmdline.set_message_viewport(new_message_viewport);
}

pub fn cmdline_clear_input(tree: &mut Tree, text_contents: &mut TextContents) {
  debug_assert!(tree.cmdline().is_some());

  let input_text = text_contents.command_line_input_mut();
  input_text.clear();

  let cmdline = tree.cmdline_mut().unwrap();
  let opts = *cmdline.options();
  let actual_size = cmdline.input().actual_shape().size();

  let new_input_viewport =
    Viewport::view(&opts, input_text, &actual_size, 0, 0);
  let new_input_cursor_viewport = CursorViewport::to_arc(
    CursorViewport::from_top_left(&new_input_viewport, input_text),
  );

  let new_input_viewport = Viewport::to_arc(new_input_viewport);
  cmdline.set_input_viewport(new_input_viewport);
  cmdline.set_input_cursor_viewport(new_input_cursor_viewport);
}
