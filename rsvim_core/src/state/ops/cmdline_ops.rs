//! Command-line operations.

use crate::content::TextContents;
use crate::ui::tree::Inodeable;
use crate::ui::tree::Tree;
use crate::ui::viewport::Viewport;
use compact_str::ToCompactString;
use ringbuf::traits::Consumer;
use ringbuf::traits::RingBuffer;

pub fn cmdline_flush_pending_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
) {
  debug_assert!(tree.command_line().is_some());

  // If message history contains some payload. This means before we actually
  // running the event loop, there's already some messages wait for print.
  let maybe_last_msg =
    text_contents.command_line_message_history().last().cloned();
  if let Some(last_msg) = maybe_last_msg {
    // Current "command-line-message" widget can only print 1 single-line
    // message, multi-line messages are not support yet.
    //
    // FIXME: Fix me once our "command-line-message" widget support
    // multi-line messages.
    let message_text = text_contents.command_line_message_mut();
    message_text.clear();
    message_text.insert_at(0, 0, last_msg.to_compact_string());

    let cmdline = tree.command_line_mut().unwrap();
    let opts = *cmdline.options();
    let actual_shape = *cmdline.message().actual_shape();

    let new_message_viewport = Viewport::to_arc(Viewport::view(
      &opts,
      message_text,
      &actual_shape,
      0,
      0,
    ));

    cmdline.set_message_viewport(new_message_viewport);
  }
}

pub fn cmdline_set_message(
  tree: &mut Tree,
  text_contents: &mut TextContents,
  payload: String,
) {
  debug_assert!(tree.command_line().is_some());

  let message_text = text_contents.command_line_message_mut();
  message_text.clear();
  message_text.insert_at(0, 0, payload.to_compact_string());

  let cmdline = tree.command_line_mut().unwrap();
  let opts = *cmdline.options();
  let actual_shape = *cmdline.message().actual_shape();

  let new_message_viewport =
    Viewport::to_arc(Viewport::view(&opts, message_text, &actual_shape, 0, 0));

  cmdline.set_message_viewport(new_message_viewport);

  // Also append message history:
  let cmdline_hist = text_contents.command_line_message_history_mut();
  cmdline_hist.push_overwrite(payload);
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
