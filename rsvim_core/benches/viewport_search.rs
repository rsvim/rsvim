//! This benchmark needs sibling repo `rsvim/tests_and_benchmarks`:
//!
//! 1. `benches/bigfiles/MIMRT1176_cm7.h`
//! 2. `dcn_3_2_0_sh_mask.h`

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use itertools::Itertools;
use ropey::Rope;
use rsvim_core::buf::Buffer;
use rsvim_core::buf::opt::BufferOptionsBuilder;
use rsvim_core::prelude::*;
use rsvim_core::ui::tree::Inodify;
use rsvim_core::ui::tree::Tree;
use rsvim_core::ui::viewport::CursorViewport;
use rsvim_core::ui::viewport::Viewport;
use rsvim_core::ui::widget::window::opt::WindowOptionsBuilder;
use std::hint::black_box;
use std::sync::Arc;
use taffy::Style;
use taffy::prelude::FromLength;
use taffy::prelude::FromPercent;

const BIG_TERM_WIDTH: u16 = 200;
const BIG_TERM_HEIGHT: u16 = 50;
const SMALL_TERM_WIDTH: u16 = 45;
const SMALL_TERM_HEIGHT: u16 = 12;
const BIG_FILE1: &str =
  "../../../tests_and_benchmarks/benches/bigfiles/dcn_3_2_0_sh_mask.h";
const BIG_FILE2: &str =
  "../../../tests_and_benchmarks/benches/bigfiles/MIMXRT1176_cm7.h";
const REPEAT: usize = 1000;

fn bench_search_nowrap_bigterm(c: &mut Criterion) {
  let canvas_size = size!(BIG_TERM_WIDTH, BIG_TERM_HEIGHT);
  let buffer_opts = BufferOptionsBuilder::default().build().unwrap();
  let window_opts =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();
  let buffers = vec![
    std::fs::read_to_string(BIG_FILE1).unwrap(),
    std::fs::read_to_string(BIG_FILE2).unwrap(),
  ]
  .iter()
  .map(|text| {
    let rp = Rope::from_str(text);
    let buf = Buffer::_new(
      buffer_opts,
      canvas_size,
      rp.clone(),
      None,
      None,
      None,
      None,
      None,
      None,
      None,
    );
    Buffer::to_arc(buf)
  })
  .collect_vec();

  for buffer in buffers.iter() {
    let tree_style = Style {
      size: taffy::Size {
        width: taffy::prelude::length(canvas_size.width()),
        height: taffy::prelude::length(canvas_size.height()),
      },
      ..Default::default()
    };
    let mut tree = Tree::new(tree_style).unwrap();
    tree.set_global_local_options(window_opts);
    let window_style = Style {
      size: taffy::Size {
        height: taffy::prelude::percent(1.0),
        width: taffy::prelude::percent(1.0),
      },
      ..Default::default()
    };
    let window_id = tree
      .new_window_with_parent(
        tree.root_id(),
        window_style,
        *tree.global_local_options(),
        Arc::downgrade(&buffer),
      )
      .unwrap();

    let buffer = lock!(buffer);

    for _i in 0..REPEAT {
      let target_cursor_line = fastrand::usize(..);
      let target_cursor_char = fastrand::usize(..);

      let old_viewport = tree.window(window_id).unwrap().viewport();
      let old_cursor_viewport =
        tree.window(window_id).unwrap().cursor_viewport();
      let (start_line, start_column) = old_viewport.search(
        &old_cursor_viewport,
        &window_opts,
        buffer.text(),
        &tree.window(window_id).unwrap().actual_shape().size(),
        target_cursor_line,
        target_cursor_char,
      );
      let new_viewport = Viewport::view(
        &window_opts,
        buffer.text(),
        &tree.window(window_id).unwrap().actual_shape().size(),
        start_line,
        start_column,
      );
      let new_cursor_viewport =
        CursorViewport::to_arc(CursorViewport::from_position(
          &new_viewport,
          buffer.text(),
          &tree.window(window_id).unwrap().actual_shape().size(),
          target_cursor_line,
          target_cursor_char,
        ));
      tree.set_editable_cursor_viewport(window_id, new_cursor_viewport);
      tree.set_editable_viewport(window_id, Viewport::to_arc(new_viewport));
    }
  }
}

fn bench_search_nowrap_smallterm(c: &mut Criterion) {
  let canvas_size = size!(BIG_TERM_WIDTH, BIG_TERM_HEIGHT);
  let buffer_opts = BufferOptionsBuilder::default().build().unwrap();
  let window_opts =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();
  let file_text = std::fs::read_to_string(BIG_FILE1);
}
