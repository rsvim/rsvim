//! Commandline's text content widget.

use crate::content::TextContentsWk;
use crate::prelude::*;
use crate::ui::canvas::Canvas;
use crate::ui::tree::*;
use crate::ui::viewport::ViewportWk;
use crate::ui::widget::Widgetable;
use crate::{inode_impl, lock};
use compact_str::CompactString;

#[derive(Debug, Clone)]
/// Commandline message.
pub struct CommandLineMessage {
  base: InodeBase,
  message_contents: TextContentsWk,
  message_viewport: ViewportWk,
}

impl CommandLineMessage {
  /// Make window content.
  pub fn new(
    shape: IRect,
    text_contents: TextContentsWk,
    message_viewport: ViewportWk,
  ) -> Self {
    let base = InodeBase::new(shape);
    CommandLineMessage {
      base,
      message_contents: text_contents,
      message_viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.message_viewport = viewport;
  }

  pub fn get_text_contents(&self) -> &TextContentsWk {
    &self.message_contents
  }

  pub fn get_text_contents_mut(&mut self) -> &mut TextContentsWk {
    &mut self.message_contents
  }

  pub fn set_message(&mut self, text: CompactString) {
    let message_content = self.get_text_contents_mut().upgrade().unwrap();
    let mut message_content = lock!(message_content);
    let message_content = message_content.command_line_message_mut();
    message_content.clear();
    message_content.insert_at(0, 0, text);
  }
}

inode_impl!(CommandLineMessage, base);

impl Widgetable for CommandLineMessage {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.message_contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.message_viewport.upgrade().unwrap();

    viewport.draw(contents.command_line_message(), actual_shape, canvas);
  }
}
