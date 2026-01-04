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

fn _set_message_impl(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: String,
) {
  debug_assert!(tree.cmdline_id().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();
  message_text.insert_at(0, 0, payload.to_compact_string());

  let (opts, message_actual_size) = {
    let cmdline = tree.cmdline().unwrap();
    let opts = *cmdline.options();
    let message_id = cmdline.message_id();
    let actual_size = match tree.node(message_id).unwrap() {
      TreeNode::CmdlineMessage(message) => message.actual_shape().size(),
      _ => unreachable!(),
    };
    (opts, actual_size)
  };

  let new_message_viewport = Viewport::to_arc(Viewport::view(
    &opts,
    message_text,
    &message_actual_size,
    0,
    0,
  ));

  tree.set_cmdline_message_viewport(new_message_viewport);
}

pub fn cmdline_set_last_pending_message_on_initialize(
  tree: &mut Tree,
  text_contents: &mut TextContents,
) {
  // If message history contains some payload. This means before we actually
  // running the event loop, there's already some messages wait for print.
  let last_msg = text_contents.command_line_message_history().last().cloned();
  trace!("|cmdline_flush_pending_message| last_msg:{:?}", last_msg);
  if let Some(last_msg) = last_msg {
    // Current "command-line-message" widget can only print 1 single-line
    // message, multi-line messages are not support yet.
    //
    // FIXME: Fix me once our "command-line-message" widget support
    // multi-line messages.
    _set_message_impl(tree, text_contents, last_msg);
  }
}

pub fn cmdline_set_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: String,
) {
  _set_message_impl(tree, text_contents, payload.clone());

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

  let (opts, message_actual_size) = {
    let cmdline = tree.cmdline().unwrap();
    let opts = *cmdline.options();
    let message_id = cmdline.message_id();
    let actual_size = match tree.node(message_id).unwrap() {
      TreeNode::CmdlineMessage(message) => message.actual_shape().size(),
      _ => unreachable!(),
    };
    (opts, actual_size)
  };

  let new_message_viewport = Viewport::to_arc(Viewport::view(
    &opts,
    message_text,
    &message_actual_size,
    0,
    0,
  ));

  tree.set_cmdline_message_viewport(new_message_viewport);
}

pub fn cmdline_clear_input(tree: &mut Tree, text_contents: &mut TextContents) {
  debug_assert!(tree.cmdline_id().is_some());

  let input_text = text_contents.command_line_input_mut();
  input_text.clear();

  let (opts, input_actual_size) = {
    let cmdline = tree.cmdline().unwrap();
    let opts = *cmdline.options();
    let input_id = cmdline.input_id();
    let actual_size = match tree.node(input_id).unwrap() {
      TreeNode::CmdlineInput(input) => input.actual_shape().size(),
      _ => unreachable!(),
    };
    (opts, actual_size)
  };

  let new_input_viewport =
    Viewport::view(&opts, input_text, &input_actual_size, 0, 0);
  let new_input_cursor_viewport =
    CursorViewport::from_top_left(&new_input_viewport, input_text);
  let new_input_viewport = Viewport::to_arc(new_input_viewport);
  let new_input_cursor_viewport =
    CursorViewport::to_arc(new_input_cursor_viewport);

  tree.set_editable_viewport(tree.cmdline_id().unwrap(), new_input_viewport);
  tree.set_editable_cursor_viewport(
    tree.cmdline_id().unwrap(),
    new_input_cursor_viewport,
  );
}
