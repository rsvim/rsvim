//! Buffer debug utils.

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
pub fn dbg_print_textline(text: &Text, line_idx: usize, char_idx: usize, msg: &str) {
  trace!("{} Text line:{},char:{}", msg, line_idx, char_idx);

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
          if i + start_char_on_line == char_idx {
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

  trace!("{} Whole text:", msg);
  for i in 0..text.rope().len_lines() {
    trace!("{i}:{:?}", ropeline_to_string(&text.rope().line(i)));
  }
}

#[cfg(not(debug_assertions))]
pub fn dbg_print_textline(buffer: &Text, line_idx: usize, char_idx: usize, msg: &str) {}
