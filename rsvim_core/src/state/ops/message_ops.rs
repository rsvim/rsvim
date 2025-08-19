use crate::ui::tree::Inodeable;
use crate::ui::viewport::Viewport;
use crate::ui::widget::command_line::CommandLine;
use crate::{geo_rect_as, lock};

/// Refresh the command line view to have no content in both content and message widgets.
pub fn refresh_view(command_line: &mut CommandLine) {
  let input_shape = command_line.input_mut().shape();
  let input_actual_shape = geo_rect_as!(input_shape, u16);
  let text_contents = command_line.text_contents().upgrade().unwrap();
  let mut text_contents = lock!(text_contents);
  let content = text_contents.command_line_input_mut();
  content.clear();
  let input_viewport =
    Viewport::view(command_line.options(), content, &input_actual_shape, 0, 0);
  let input_viewport_arc = Viewport::to_arc(input_viewport);
  let message = text_contents.command_line_message_mut();
  message.clear();
  command_line.set_input_viewport(input_viewport_arc.clone());
  let message_viewport =
    Viewport::view(command_line.options(), message, &input_actual_shape, 0, 0);
  let message_viewport_arc = Viewport::to_arc(message_viewport);
  command_line.set_message_viewport(message_viewport_arc.clone());
}
