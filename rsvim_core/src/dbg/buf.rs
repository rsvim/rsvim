//! Buffer debug utils.

#![allow(unused_imports)]

use crate::buf::text::Text;

use tracing::trace;

pub fn ropeline_to_string(bufline: &ropey::RopeSlice) -> String {
  let mut builder = String::with_capacity(bufline.len_chars());
  for c in bufline.chars() {
    builder.push(c);
  }
  builder
}

#[cfg(debug_assertions)]
pub fn dbg_print_textline_with_absolute_char_idx(
  text: &Text,
  line_idx: usize,
  absolute_char_idx: usize,
  msg: &str,
) {
  trace!(
    "{} text line:{},absolute_char:{}",
    msg, line_idx, absolute_char_idx
  );

  match text.rope().get_line(line_idx) {
    Some(line) => {
      trace!("len_chars:{}", line.len_chars());
      let start_char_on_line = text.rope().line_to_char(line_idx);

      let mut builder1 = String::new();
      let mut builder2 = String::new();
      for (i, c) in line.chars().enumerate() {
        let w = text.char_width(c);
        if w > 0 {
          builder1.push(c);
        }
        let s: String = std::iter::repeat_n(
          if i + start_char_on_line == absolute_char_idx {
            '^'
          } else {
            ' '
          },
          w,
        )
        .collect();
        builder2.push_str(s.as_str());
      }
      trace!("-{}-", builder1);
      trace!("-{}-", builder2);
    }
    None => trace!("line not exist"),
  }

  trace!("{} whole text:", msg);
  for i in 0..text.rope().len_lines() {
    trace!("{i}:{:?}", ropeline_to_string(&text.rope().line(i)));
  }
}

#[cfg(not(debug_assertions))]
pub fn dbg_print_textline_with_absolute_char_idx(
  _text: &Text,
  _line_idx: usize,
  _char_idx: usize,
  _msg: &str,
) {
}

#[cfg(debug_assertions)]
pub fn dbg_print_textline(text: &Text, line_idx: usize, char_idx: usize, msg: &str) {
  trace!("{} text line:{},char:{}", msg, line_idx, char_idx);

  match text.rope().get_line(line_idx) {
    Some(bufline) => {
      trace!("len_chars:{}", bufline.len_chars());
      let mut builder1 = String::new();
      let mut builder2 = String::new();
      for (i, c) in bufline.chars().enumerate() {
        let w = text.char_width(c);
        if w > 0 {
          builder1.push(c);
        }
        let s: String = std::iter::repeat_n(if i == char_idx { '^' } else { ' ' }, w).collect();
        builder2.push_str(s.as_str());
      }
      trace!("-{}-", builder1);
      trace!("-{}-", builder2);
    }
    None => trace!("line not exist"),
  }

  trace!("{}, whole buffer:", msg);
  for i in 0..text.rope().len_lines() {
    trace!("{i}:{:?}", ropeline_to_string(&text.rope().line(i)));
  }
}

#[cfg(not(debug_assertions))]
pub fn dbg_print_textline(_text: &Text, _line_idx: usize, _char_idx: usize, _msg: &str) {}
