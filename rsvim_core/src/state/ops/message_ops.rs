use crate::ui::tree::Inodeable;
use crate::ui::widget::command_line::CommandLine;

/// Toggles visibility between the command-line content and its message.
///
/// - If `visible` is true: show the message and hide the content.
/// - If `visible` is false: hide the message and show the content.
pub fn set_message_visible(command_line: &mut CommandLine, visible: bool) {
  command_line.indicator_mut().set_visible(!visible);
  command_line.input_mut().set_visible(!visible);
  command_line.message_mut().set_visible(visible);
}
