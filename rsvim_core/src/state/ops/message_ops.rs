use crate::state::fsm::StatefulDataAccess;
use crate::state::ops::cursor_ops::_update_viewport_after_text_changed;
use crate::ui::tree::Inodeable;
use crate::ui::viewport::Viewport;
use crate::ui::widget::command_line::CommandLine;
use crate::{geo_rect_as, lock};

/// If the message widget has received any text, it prints it in the widget.
pub fn may_show_message(data_access: &StatefulDataAccess) {
  {
    let tree = data_access.tree.clone();
    let mut tree = lock!(tree);
    let mut tree_clone = tree.clone();
    let command_line = tree_clone.command_line_mut().unwrap();
    let message = command_line.message_mut();
    if let Some(text) = message.get_message() {
      let text_contents = message.get_text_contents_mut().upgrade().unwrap();
      let mut text_contents = lock!(text_contents);
      let message_content = text_contents.command_line_content_mut();
      message_content.clear();
      message_content.insert_at(0, 0, text);
      _update_viewport_after_text_changed(
        &mut tree,
        command_line.id(),
        message_content,
      );
    };
  }
}

/// Toggles visibility between the command-line content and its message.
///
/// - If `visible` is true: show the message and hide the content.
/// - If `visible` is false: hide the message and show the content.
pub fn set_message_visible(command_line: &mut CommandLine, visible: bool) {
  command_line.content_mut().set_visible(!visible);
  command_line.message_mut().set_visible(visible);
  let message_contents = command_line
    .message_mut()
    .get_text_contents_mut()
    .upgrade()
    .unwrap();
  let mut message_contents = lock!(message_contents);
  message_contents.command_line_content_mut().clear();
  if !visible {
    command_line.message_mut().set_message(None);
  }
}

/// Refresh the command line view to have no content in both content and message widgets.
pub fn refresh_view(command_line: &mut CommandLine) {
  let cmdline_content_shape = command_line.content_mut().shape();
  let cmdline_content_shape = geo_rect_as!(cmdline_content_shape, u16);
  let content = command_line.text_contents().upgrade().unwrap();
  let mut content = lock!(content);
  let content = content.command_line_content_mut();
  content.clear();
  let message_viewport = Viewport::view(
    command_line.options(),
    content,
    &cmdline_content_shape,
    0,
    0,
  );
  let message_viewport_arc = Viewport::to_arc(message_viewport);
  command_line.set_message_viewport(message_viewport_arc.clone());
  let content_viewport = Viewport::view(
    command_line.options(),
    content,
    &cmdline_content_shape,
    0,
    0,
  );
  let content_viewport_arc = Viewport::to_arc(content_viewport);
  command_line.set_content_viewport(content_viewport_arc.clone());
}
