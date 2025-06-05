//! Buffer utils for testing.

#![allow(unused_imports)]

use crate::buf::{Buffer, BufferArc, BufferLocalOptions, BuffersManager, BuffersManagerArc};
//use crate::envar;
use crate::lock;

use ropey::{Rope, RopeBuilder, RopeSlice};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::BufReader;
use tracing::{self, info};
use tracing_appender::non_blocking::DEFAULT_BUFFERED_LINES_LIMIT;

/// Create rope from filename.
pub fn make_rope_from_file(filename: String) -> Rope {
  Rope::from_reader(BufReader::new(File::open(filename).unwrap())).unwrap()
}

/// Create rope from lines.
pub fn make_rope_from_lines(lines: Vec<&str>) -> Rope {
  let mut rb: RopeBuilder = RopeBuilder::new();
  for line in lines.iter() {
    rb.append(line);
  }
  rb.finish()
}

pub fn make_buffer_from_file(
  terminal_height: u16,
  opts: BufferLocalOptions,
  filename: String,
) -> BufferArc {
  let rp = make_rope_from_file(filename);
  let bf = Buffer::_new(terminal_height, rp, opts, None, None, None, None);
  Buffer::to_arc(bf)
}

pub fn make_buffer_from_lines(
  terminal_height: u16,
  opts: BufferLocalOptions,
  lines: Vec<&str>,
) -> BufferArc {
  let rp = make_rope_from_lines(lines);
  let buf = Buffer::_new(terminal_height, rp, opts, None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_empty_buffer(terminal_height: u16, opts: BufferLocalOptions) -> BufferArc {
  let buf = Buffer::_new_empty(terminal_height, opts);
  Buffer::to_arc(buf)
}

pub fn make_buffer_from_rope(
  terminal_height: u16,
  opts: BufferLocalOptions,
  rp: Rope,
) -> BufferArc {
  let buf = Buffer::_new(terminal_height, rp, opts, None, None, None, None);
  Buffer::to_arc(buf)
}

pub fn make_buffers_manager(opts: BufferLocalOptions, bufs: Vec<BufferArc>) -> BuffersManagerArc {
  let mut bm = BuffersManager::new();
  bm.set_global_local_options(&opts);
  for buf in bufs.iter() {
    bm._add_buffer(buf.clone());
  }
  BuffersManager::to_arc(bm)
}

#[allow(clippy::unused_enumerate_index)]
pub fn print_buffer_line_details(buf: BufferArc, line_idx: usize, msg: &str) {
  let buf = lock!(buf);
  let line = buf.get_rope().get_line(line_idx).unwrap();

  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .with_line_number(false)
    .with_target(false)
    .with_level(true)
    .with_ansi(true)
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .with_writer(std::io::stdout)
    .without_time()
    .finish();

  tracing::subscriber::with_default(subscriber, || {
    if !msg.is_empty() {
      info!("line: {}", msg);
    } else {
      info!("line");
    }

    let mut payload = String::new();
    for c in line.chars() {
      let (cs, _cw) = buf.char_symbol(c);
      payload.push_str(cs.as_ref());
    }
    info!("-{}-", payload);

    {
      let mut builder = String::new();
      let mut n = 0_usize;
      let mut w = 0_usize;
      let mut zero_width_chars: Vec<String> = vec![];
      let mut big_width_chars: Vec<String> = vec![];
      for (i, c) in line.chars().enumerate() {
        let (_cs, cw) = buf.char_symbol(c);
        w += cw;
        n += 1;
        if cw == 0 {
          zero_width_chars.push(format!("{}", i));
        }
        if cw > 1 {
          big_width_chars.push(format!("{}", i));
        }
        if i % 5 == 0 {
          builder.push_str(&format!("{}", i));
        }
        if builder.len() < w {
          let diff = w - builder.len();
          builder.push_str(&" ".repeat(diff));
        }
      }
      info!(
        "-{}- Char Index, total:{} (width = 0 chars: count:{} indexes:{}, width > 1 chars: count:{} indexes:{})",
        builder,
        n,
        zero_width_chars.len(),
        zero_width_chars.join(","),
        big_width_chars.len(),
        big_width_chars.join(",")
      );
    }

    {
      let mut builder1 = String::new();
      let mut builder2 = String::new();
      let mut builder3 = String::new();
      let mut show2 = false;
      let mut show3 = false;
      let mut w = 0_usize;
      for (_i, c) in line.chars().enumerate() {
        let (_cs, cw) = buf.char_symbol(c);
        w += cw;
        if w == 1 || w % 5 == 0 {
          if builder1.is_empty() || builder1.ends_with(" ") {
            builder1.push_str(&format!("{}", w));
          } else if cw > 0 {
            builder2.push_str(&format!("{}", w));
            show2 = true;
          } else {
            builder3.push_str(&format!("{}", w));
            show3 = true;
          }
        }

        if builder1.len() < w {
          let diff = w - builder1.len();
          builder1.push_str(&" ".repeat(diff));
        }
        if builder2.len() < w {
          let diff = w - builder2.len();
          builder2.push_str(&" ".repeat(diff));
        }
        if builder3.len() < w {
          let diff = w - builder3.len();
          builder3.push_str(&" ".repeat(diff));
        }
      }
      info!("-{}- Display Width, total width:{}", builder1, w);
      if show2 {
        info!(
          "-{}- Display Width (extra, conflicted with the above one)",
          builder2
        );
      }
      if show3 {
        info!("-{}- Display Width for width = 0 chars", builder3);
      }
    }

    {
      let mut builder = String::new();
      let mut w = 0_usize;
      let mut show = false;
      for (_i, c) in line.chars().enumerate() {
        let (_cs, cw) = buf.char_symbol(c);
        w += cw;
        if cw > 1 && (builder.is_empty() || builder.ends_with(" ")) {
          builder.push_str(&" ".repeat(cw - 1));
          builder.push_str(&format!("{}", w));
          show = true;
        }

        if builder.len() < w {
          let diff = w - builder.len();
          builder.push_str(&" ".repeat(diff));
        }
      }
      if show {
        info!("-{}- Display Width for width > 1 chars", builder);
      }
    }
  });
}

pub fn bufline_to_string(bufline: &RopeSlice) -> String {
  let mut builder = String::with_capacity(bufline.len_chars());
  for c in bufline.chars() {
    builder.push(c);
  }
  builder
}

pub fn print_buffer(buf: &Buffer, msg: &str) {
  info!("{} whole buffer:", msg);
  for i in 0..buf.get_rope().len_lines() {
    info!("{i}:{:?}", bufline_to_string(&buf.get_rope().line(i)));
  }
}

pub fn print_bufline_and_focus_char_on_line(
  buffer: &Buffer,
  line_idx: usize,
  char_idx: usize,
  msg: &str,
) {
  match buffer.get_rope().get_line(line_idx) {
    Some(bufline) => {
      info!(
        "{} line:{}, len_chars:{}, focus char:{}",
        msg,
        line_idx,
        bufline.len_chars(),
        char_idx
      );
      let mut builder1 = String::new();
      let mut builder2 = String::new();
      for (i, c) in bufline.chars().enumerate() {
        let w = buffer.char_width(c);
        if w > 0 {
          builder1.push(c);
        }
        let s: String = std::iter::repeat(if i == char_idx { '^' } else { ' ' })
          .take(w)
          .collect();
        builder2.push_str(s.as_str());
      }
      info!("-{}-", builder1);
      info!("-{}-", builder2);
    }
    None => info!(
      "{} line:{}, focus char:{}, not exist",
      msg, line_idx, char_idx
    ),
  }
}

pub fn print_bufline_and_focus_char(buffer: &Buffer, line_idx: usize, char_idx: usize, msg: &str) {
  match buffer.get_rope().get_line(line_idx) {
    Some(bufline) => {
      info!(
        "{} line:{}, len_chars:{}, focus char:{}",
        msg,
        line_idx,
        bufline.len_chars(),
        char_idx
      );
      let start_char_on_line = buffer.get_rope().line_to_char(line_idx);

      let mut builder1 = String::new();
      let mut builder2 = String::new();
      for (i, c) in bufline.chars().enumerate() {
        let w = buffer.char_width(c);
        if w > 0 {
          builder1.push(c);
        }
        let s: String = std::iter::repeat(if i + start_char_on_line == char_idx {
          '^'
        } else {
          ' '
        })
        .take(w)
        .collect();
        builder2.push_str(s.as_str());
      }
      info!("-{}-", builder1);
      info!("-{}-", builder2);
    }
    None => info!(
      "{} line:{}, focus char:{}, not exist",
      msg, line_idx, char_idx
    ),
  }
}
