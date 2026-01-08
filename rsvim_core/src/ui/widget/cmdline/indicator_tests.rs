#![allow(unused_imports)]

use super::indicator::*;
use crate::buf::BufferArc;
use crate::buf::opt::BufferOptions;
use crate::buf::opt::BufferOptionsBuilder;
use crate::prelude::*;
use crate::tests::buf::make_buffer_from_lines;
use crate::tests::buf::make_empty_buffer;
use crate::tests::log::init as test_log_init;
use crate::tests::viewport::assert_canvas;
use crate::ui::canvas::Canvas;
use crate::ui::tree::Tree;
use crate::ui::tree::TreeContext;
use crate::ui::viewport::Viewport;
use crate::ui::viewport::ViewportArc;
use crate::ui::widget::Widgetable;
use crate::ui::widget::window::opt::WindowOptions;
use crate::ui::widget::window::opt::WindowOptionsBuilder;
use compact_str::ToCompactString;
use ropey::Rope;
use ropey::RopeBuilder;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::rc::Rc;
use std::sync::Arc;
use taffy::Style;

fn make_canvas(
  terminal_size: U16Size,
  cmdline_indicator: &CmdlineIndicator,
) -> Canvas {
  let mut canvas = Canvas::new(terminal_size);
  cmdline_indicator.draw(&mut canvas);
  canvas
}

#[cfg(test)]
mod tests_nowrap {
  use super::*;

  #[test]
  fn new1() {
    test_log_init();

    let ctx = TreeContext::to_rc(TreeContext::new());
    let id = ctx
      .borrow_mut()
      .new_leaf_default(
        Style {
          size: taffy::Size {
            height: taffy::prelude::length(1_u16),
            width: taffy::prelude::length(1_u16),
          },
          ..Default::default()
        },
        "CmdlineIndicator",
      )
      .unwrap();
    let root_id = ctx.borrow().root();
    ctx.borrow_mut().compute_layout(root_id).unwrap();

    let terminal_size = size!(1, 1);
    let expect = vec![":"];

    let cmdline_indicator = CmdlineIndicator::new(
      id,
      Rc::downgrade(&ctx),
      CmdlineIndicatorSymbol::Colon,
    );
    let actual = make_canvas(terminal_size, &cmdline_indicator);
    assert_canvas(&actual, &expect);
  }
}
