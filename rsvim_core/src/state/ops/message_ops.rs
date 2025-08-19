use crate::ui::tree::Inodeable;
use crate::ui::viewport::Viewport;
use crate::ui::widget::command_line::CommandLine;
use crate::{geo_rect_as, lock};

/// Toggles visibility between the command-line content and its message.
///
/// - If `visible` is true: show the message and hide the content.
/// - If `visible` is false: hide the message and show the content.
pub fn set_message_visible(command_line: &mut CommandLine, visible: bool) {
  command_line.indicator_mut().set_visible(!visible);
  command_line.input_mut().set_visible(!visible);
  command_line.message_mut().set_visible(visible);
  let text_contents = command_line
    .message_mut()
    .get_text_contents_mut()
    .upgrade()
    .unwrap();
  let mut text_contents = lock!(text_contents);
  text_contents.command_line_input_mut().clear();
}

/// Refresh the command line view to have no content in both content and message widgets.
pub fn refresh_view(command_line: &mut CommandLine) {
  let input_shape = command_line.input_mut().shape();
  let cmdline_content_shape = geo_rect_as!(input_shape, u16);
  let text_contents = command_line.text_contents().upgrade().unwrap();
  let mut text_contents = lock!(text_contents);
  let content = text_contents.command_line_input_mut();
  content.clear();
  let content_viewport = Viewport::view(
    command_line.options(),
    content,
    &cmdline_content_shape,
    0,
    0,
  );
  let content_viewport_arc = Viewport::to_arc(content_viewport);
  let message = text_contents.command_line_message_mut();
  message.clear();
  command_line.set_input_viewport(content_viewport_arc.clone());
  let message_viewport = Viewport::view(
    command_line.options(),
    message,
    &cmdline_content_shape,
    0,
    0,
  );
  let message_viewport_arc = Viewport::to_arc(message_viewport);
  command_line.set_message_viewport(message_viewport_arc.clone());
}
