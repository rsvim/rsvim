//! This benchmark needs sibling repo `rsvim/tests_and_benchmarks`:
//!
//! 1. `benches/bigfiles/MIMRT1176_cm7.h`
//! 2. `dcn_3_2_0_sh_mask.h`

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use ropey::Rope;
use rsvim_core::buf::Buffer;
use rsvim_core::buf::BufferArc;
use rsvim_core::buf::opt::BufferOptions;
use rsvim_core::buf::opt::BufferOptionsBuilder;
use rsvim_core::prelude::*;
use rsvim_core::ui::tree::Inodify;
use rsvim_core::ui::tree::NodeId;
use rsvim_core::ui::tree::Tree;
use rsvim_core::ui::viewport::CursorViewport;
use rsvim_core::ui::viewport::Viewport;
use rsvim_core::ui::widget::window::opt::WindowOptions;
use rsvim_core::ui::widget::window::opt::WindowOptionsBuilder;
use std::hint::black_box;
use std::sync::Arc;
use taffy::Style;

const BIG_TERM_WIDTH: u16 = 200;
const BIG_TERM_HEIGHT: u16 = 50;
const SMALL_TERM_WIDTH: u16 = 45;
const SMALL_TERM_HEIGHT: u16 = 12;
const FILENAME1: &str = "dcn_3_2_0_sh_mask.h";
const FILENAME2: &str = "MIMXRT1176_cm7.h";
const FILETEXT1: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tests_and_benchmarks/benches/bigfiles/MIMXRT1176_cm7.h"
));
const FILETEXT2: &str = include_str!(concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tests_and_benchmarks/benches/bigfiles/dcn_3_2_0_sh_mask.h"
));
const REPEAT: usize = 100;

fn make_buffer(
  filetext: &str,
  buffer_opts: BufferOptions,
  canvas_size: U16Size,
) -> BufferArc {
  let rop = Rope::from_str(filetext);
  let buffer = Buffer::_new(
    buffer_opts,
    canvas_size,
    rop,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
  );
  Buffer::to_arc(buffer)
}

fn make_tree(
  canvas_size: &U16Size,
  window_opts: WindowOptions,
  buffer: &BufferArc,
) -> (Tree, NodeId) {
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
      Arc::downgrade(buffer),
    )
    .unwrap();
  (tree, window_id)
}

fn bench_search_nowrap(c: &mut Criterion) {
  let buffer_opts = BufferOptionsBuilder::default().build().unwrap();
  let window_opts =
    WindowOptionsBuilder::default().wrap(false).build().unwrap();

  let mut g = c.benchmark_group("bench_search_nowrap");

  let run_bench = |width: &u16, height: &u16, filetext: &str| {
    let canvas_size = size!(*width, *height);
    let buffer = make_buffer(filetext, buffer_opts, canvas_size);
    let (mut tree, window_id) = make_tree(&canvas_size, window_opts, &buffer);
    let buffer = lock!(buffer);
    for _i in 0..REPEAT {
      let target_cursor_line = fastrand::usize(..);
      let target_cursor_char = fastrand::usize(..);
      let target_cursor_line =
        target_cursor_line % buffer.text().rope().len_lines();
      let target_cursor_char = std::cmp::min(
        buffer.text().rope().line(target_cursor_line).len_chars(),
        target_cursor_char
          % (buffer
            .text()
            .last_char_idx_on_line_exclude_eol(target_cursor_line)
            .unwrap_or(0)
            + 2),
      );

      let old_viewport = tree.window(window_id).unwrap().viewport();
      let old_cursor_viewport =
        tree.window(window_id).unwrap().cursor_viewport();
      let (start_line, start_column) = old_viewport.search(
        black_box(&old_cursor_viewport),
        black_box(&window_opts),
        black_box(buffer.text()),
        black_box(&tree.window(window_id).unwrap().actual_shape().size()),
        black_box(target_cursor_line),
        black_box(target_cursor_char),
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
  };

  for canvas_width in [BIG_TERM_WIDTH, SMALL_TERM_WIDTH] {
    for canvas_height in [BIG_TERM_HEIGHT, SMALL_TERM_HEIGHT] {
      for (filename, filetext) in
        [(FILENAME1, FILETEXT1), (FILENAME2, FILETEXT2)]
      {
        let benchmark_id = format!(
          "Viewport::search wrap=false (width/height={}/{} file={})",
          canvas_width, canvas_height, filename
        );
        let benchmark_id_param =
          format!("{}/{}/{}", canvas_width, canvas_height, filename);
        let benchmark_id = BenchmarkId::new(&benchmark_id, &benchmark_id_param);
        let params = (canvas_width, canvas_height, filetext);
        g.bench_with_input(
          benchmark_id,
          &params,
          |b, (canvas_width_param, canvas_height_param, filetext_param)| {
            b.iter(|| {
              run_bench(canvas_width_param, canvas_height_param, filetext_param)
            })
          },
        );
      }
    }
  }
}

criterion_group!(benches, bench_search_nowrap);
criterion_main!(benches);
