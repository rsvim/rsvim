//! Commandline's message widget.

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
pub struct Message {
  base: InodeBase,
  text_contents: TextContentsWk,
  viewport: ViewportWk,
}

impl Message {
  pub fn new(
    shape: IRect,
    text_contents: TextContentsWk,
    viewport: ViewportWk,
  ) -> Self {
    let base = InodeBase::new(shape);
    Message {
      base,
      text_contents,
      viewport,
    }
  }

  pub fn set_viewport(&mut self, viewport: ViewportWk) {
    self.viewport = viewport;
  }

  pub fn get_text_contents(&self) -> &TextContentsWk {
    &self.text_contents
  }

  pub fn get_text_contents_mut(&mut self) -> &mut TextContentsWk {
    &mut self.text_contents
  }

  pub fn set_message(&mut self, text: CompactString) {
    let message_content = self.get_text_contents_mut().upgrade().unwrap();
    let mut message_content = lock!(message_content);
    let message_content = message_content.command_line_message_mut();
    message_content.clear();
    message_content.insert_at(0, 0, text);
  }
}

inode_impl!(Message, base);

impl Widgetable for Message {
  fn draw(&self, canvas: &mut Canvas) {
    let actual_shape = self.actual_shape();
    let contents = self.text_contents.upgrade().unwrap();
    let contents = lock!(contents);
    let viewport = self.viewport.upgrade().unwrap();

    viewport.draw(contents.command_line_message(), actual_shape, canvas);
  }
}
