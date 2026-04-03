use compact_str::ToCompactString;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use crossterm::style::Attributes;
use crossterm::style::Color;
use rsvim_core::buf::opt::BufferOptionsBuilder;
use rsvim_core::prelude::*;
use rsvim_core::ui::canvas::Canvas;
use rsvim_core::ui::canvas::Cell;
use rsvim_core::ui::widget::window::opt::WindowOptionsBuilder;
use std::hint::black_box;
use std::time::Duration;

const BIG_TERM_WIDTH: u16 = 200;
const BIG_TERM_HEIGHT: u16 = 50;
const SMALL_TERM_WIDTH: u16 = 45;
const SMALL_TERM_HEIGHT: u16 = 12;
const REPEAT: usize = 100;
const BENCH_MEASUREMENT_TIME: Duration = Duration::from_secs(10);

fn bench_shade(c: &mut Criterion) {
  let mut g = c.benchmark_group("bench_search_nowrap");

  let run_bench = |width: &u16, height: &u16| {
    let canvas_size = size!(*width, *height);
    let mut canvas = Canvas::new(canvas_size);

    for _i in 0..REPEAT {
      let n = (*width as usize) * (*height as usize);
      for j in 0..n {
        let s = fastrand::u8(32..127) as char; // Printable chars
        let pos = canvas.frame().idx2pos(j);
        let cell = Cell::new(
          s.to_compact_string(),
          Color::White,
          Color::Black,
          Attributes::none(),
        );
        canvas.frame_mut().set_cell(black_box(pos), black_box(cell));
      }
      let shaders = canvas.shade();
      let _shaders = shaders.lock().unwrap();
    }
  };

  for canvas_width in [BIG_TERM_WIDTH, SMALL_TERM_WIDTH] {
    for canvas_height in [BIG_TERM_HEIGHT, SMALL_TERM_HEIGHT] {
      let benchmark_id_param = format!("{}/{}", canvas_width, canvas_height);
      let benchmark_id = BenchmarkId::new("shade", &benchmark_id_param);
      let params = (canvas_width, canvas_height);
      g.measurement_time(BENCH_MEASUREMENT_TIME).bench_with_input(
        benchmark_id,
        &params,
        |b, (canvas_width_param, canvas_height_param)| {
          b.iter(|| run_bench(canvas_width_param, canvas_height_param))
        },
      );
    }
  }
}

criterion_group!(benches, bench_shade);
criterion_main!(benches);
