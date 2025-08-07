use crate::content::TextContentsWk;
use crate::coord::IRect;
use crate::ui::tree::{Inodeable, Itree};
use crate::ui::viewport::{CursorViewport, Viewport};
use crate::ui::widget::command_line::content::CommandLineContent;
use crate::ui::widget::command_line::indicator::CommandLineIndicator;
use crate::ui::widget::command_line::root::CommandLineRootContainer;
use crate::ui::widget::command_line::{
  CommandLine, CommandLineIndicatorSymbol, CommandLineNode,
};
use crate::ui::widget::window::WindowLocalOptionsBuilder;
use crate::{geo_rect_as, lock};
use std::sync::Arc;

pub struct CommandLineBuilder {
  shape: IRect,
  text_contents: TextContentsWk,
  command_line_indicator_symbol: CommandLineIndicatorSymbol,
}

impl Default for CommandLineBuilder {
  fn default() -> Self {
    Self {
      shape: IRect::new((0, 0), (1, 1)),
      text_contents: TextContentsWk::new(),
      command_line_indicator_symbol: CommandLineIndicatorSymbol::Empty,
    }
  }
}

impl CommandLineBuilder {
  pub fn with_shape(&mut self, shape: IRect) -> &mut Self {
    self.shape = shape;
    self
  }

  pub fn with_text_contents(
    &mut self,
    text_contents: TextContentsWk,
  ) -> &mut Self {
    self.text_contents = text_contents;
    self
  }

  pub fn with_command_line_indicator_symbol(
    &mut self,
    command_line_indicator_symbol: CommandLineIndicatorSymbol,
  ) -> &mut Self {
    self.command_line_indicator_symbol = command_line_indicator_symbol;
    self
  }

  pub fn build(&self) -> CommandLine {
    let options = WindowLocalOptionsBuilder::default()
      .wrap(false)
      .line_break(false)
      .scroll_off(0_u16)
      .build()
      .unwrap();

    let cmdline_root = CommandLineRootContainer::new(self.shape);
    let cmdline_root_id = cmdline_root.id();
    let cmdline_root_node =
      CommandLineNode::CommandLineRootContainer(cmdline_root);

    let mut base = Itree::new(cmdline_root_node);

    let cmdline_indicator_shape = IRect::new(
      self.shape.min().into(),
      (self.shape.min().x, self.shape.max().y),
    );
    let cmdline_indicator = CommandLineIndicator::new(
      cmdline_indicator_shape,
      CommandLineIndicatorSymbol::Empty,
    );
    let cmdline_indicator_id = cmdline_indicator.id();
    let cmdline_indicator_node =
      CommandLineNode::CommandLineIndicator(cmdline_indicator);
    base.bounded_insert(cmdline_root_id, cmdline_indicator_node);

    let x_offset = if self.command_line_indicator_symbol
      == CommandLineIndicatorSymbol::Empty
    {
      0
    } else {
      1
    };
    let cmdline_content_shape = IRect::new(
      (self.shape.min().x + x_offset, self.shape.min().y),
      self.shape.max().into(),
    );

    let (viewport, cursor_viewport) = {
      let cmdline_content_actual_shape =
        geo_rect_as!(cmdline_content_shape, u16);
      let text_contents = self.text_contents.upgrade().unwrap();
      let text_contents = lock!(text_contents);
      let viewport = Viewport::view(
        &options,
        text_contents.command_line_content(),
        &cmdline_content_actual_shape,
        0,
        0,
      );
      let cursor_viewport = CursorViewport::from_top_left(
        &viewport,
        text_contents.command_line_content(),
      );
      (viewport, cursor_viewport)
    };
    let viewport = Viewport::to_arc(viewport);
    let cursor_viewport = CursorViewport::to_arc(cursor_viewport);

    let cmdline_content = CommandLineContent::new(
      cmdline_content_shape,
      self.text_contents.clone(),
      Arc::downgrade(&viewport),
    );
    let cmdline_content_id = cmdline_content.id();
    let cmdline_content_node =
      CommandLineNode::CommandLineContent(cmdline_content);
    base.bounded_insert(cmdline_root_id, cmdline_content_node);

    CommandLine {
      base,
      options,
      indicator_id: cmdline_indicator_id,
      content_id: cmdline_content_id,
      cursor_id: None,
      text_contents: self.text_contents.clone(),
      viewport,
      cursor_viewport,
    }
  }

  pub fn message_command_line() -> CommandLine {
    todo!();
  }
}
