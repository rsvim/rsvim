//! Text content backend for buffer.

pub mod cidx;

#[cfg(test)]
mod cidx_tests;

use crate::buf::opt::BufferOptions;
use crate::buf::opt::EndOfLineOption;
use crate::buf::unicode;
use crate::prelude::*;
pub use cidx::ColumnIndex;
use compact_str::CompactString;
use compact_str::ToCompactString;
use lru::LruCache;
use ropey::Rope;
use ropey::RopeSlice;
use std::cell::RefCell;
use std::rc::Rc;

// Cached lines width.
type CachedWidth = LruCache<usize, ColumnIndex, RandomState>;

#[derive(Hash, PartialEq, Eq, Copy, Clone)]
struct ClonedLineKey {
  pub line_idx: usize,
  pub start_char_idx: usize,
  pub max_chars: usize,
}
// Cached cloned lines.
type CachedClones = LruCache<ClonedLineKey, Rc<String>, RandomState>;

#[derive(Debug, Default)]
struct CacheStatus {
  hits: usize,
  misses: usize,
}

impl CacheStatus {
  pub fn hit_one(&mut self) {
    self.hits += 1;
  }

  pub fn miss_one(&mut self) {
    self.hits += 1;
  }

  pub fn hits(&self) -> usize {
    self.hits
  }

  pub fn misses(&self) -> usize {
    self.misses
  }

  pub fn total(&self) -> usize {
    self.hits + self.misses
  }
}

#[derive(Debug)]
/// Text content backend.
pub struct Text {
  rope: Rope,
  options: BufferOptions,

  // Caches for:
  // 1. Lines width
  // 2. Cloned lines, this is only used when `wrap=true,line_break=true`.
  cached_width: RefCell<LruCache<usize, ColumnIndex, RandomState>>,
  cached_clones: RefCell<LruCache<ClonedLineKey, Rc<String>, RandomState>>,
  cached_width_stats: RefCell<CacheStatus>,
  cached_clones_stats: RefCell<CacheStatus>,
}

arc_mutex_ptr!(Text);

fn _cached_size(canvas_size: U16Size) -> std::num::NonZeroUsize {
  std::num::NonZeroUsize::new(canvas_size.height() as usize * 2 + 3).unwrap()
}

impl Text {
  pub fn new(opts: BufferOptions, canvas_size: U16Size, rope: Rope) -> Self {
    let cache_size = _cached_size(canvas_size);
    Self {
      rope,
      options: opts,
      cached_width: RefCell::new(LruCache::with_hasher(
        cache_size,
        RandomState::default(),
      )),
      cached_clones: RefCell::new(LruCache::with_hasher(
        cache_size,
        RandomState::default(),
      )),
      cached_width_stats: RefCell::new(CacheStatus::default()),
      cached_clones_stats: RefCell::new(CacheStatus::default()),
    }
  }

  fn with_cached_width<F, U>(&self, f: F) -> U
  where
    F: FnOnce(&mut CachedWidth, &mut CacheStatus) -> U,
  {
    f(
      &mut self.cached_width.borrow_mut(),
      &mut self.cached_width_stats.borrow_mut(),
    )
  }

  fn with_cached_clones<F, U>(&self, f: F) -> U
  where
    F: FnOnce(&mut CachedClones, &mut CacheStatus) -> U,
  {
    f(
      &mut self.cached_clones.borrow_mut(),
      &mut self.cached_clones_stats.borrow_mut(),
    )
  }

  fn cached_width_upsert<'a, F>(
    &self,
    caches: &'a mut CachedWidth,
    stats: &mut CacheStatus,
    k: &usize,
    f: F,
  ) -> &'a mut ColumnIndex
  where
    F: FnOnce() -> ColumnIndex,
  {
    if !caches.contains(k) {
      let v = f();
      caches.put(*k, v);
      stats.miss_one();
    } else {
      stats.hit_one();
    }

    caches.get_mut(k).unwrap()
  }

  fn cached_clones_upsert<'a, F>(
    &self,
    caches: &'a mut CachedClones,
    stats: &mut CacheStatus,
    k: &ClonedLineKey,
    f: F,
  ) -> &'a Rc<String>
  where
    F: FnOnce() -> Rc<String>,
  {
    if !caches.contains(k) {
      let v = f();
      caches.put(*k, v);
      stats.miss_one();
    } else {
      stats.hit_one();
    }

    caches.get(k).unwrap()
  }
}

#[cfg(debug_assertions)]
impl Drop for Text {
  fn drop(&mut self) {
    let stats1 = self.cached_width_stats.borrow();
    trace!(
      "Text cached_lines_width - total:{},hits:{},misses:{},ratio:{}",
      stats1.total(),
      stats1.hits(),
      stats1.misses(),
      stats1.hits() as f32 / stats1.total() as f32
    );
    let stats2 = self.cached_clones_stats.borrow();
    trace!(
      "Text cached_cloned_lines - total:{},hits:{},misses:{},ratio:{}",
      stats2.total(),
      stats2.hits(),
      stats2.misses(),
      stats2.hits() as f32 / stats2.total() as f32
    );
  }
}

// Unicode {
impl Text {
  /// Get the display width for a `char`, supports both ASCI control codes and unicode.
  ///
  /// The char display width follows the
  /// [Unicode Standard Annex #11](https://www.unicode.org/reports/tr11/).
  pub fn char_width(&self, c: char) -> usize {
    unicode::char_width(&self.options, c)
  }

  /// Get the printable cell symbol.
  pub fn char_symbol(&self, c: char) -> CompactString {
    unicode::char_symbol(&self.options, c)
  }

  /// Get both cell symbol and its display width.
  pub fn char_symbol_and_width(&self, c: char) -> (CompactString, usize) {
    (
      unicode::char_symbol(&self.options, c),
      unicode::char_width(&self.options, c),
    )
  }
}
// Unicode }

// Rope {
impl Text {
  /// Get rope.
  pub fn rope(&self) -> &Rope {
    &self.rope
  }

  // Get mutable rope.
  //
  // NOTE:
  // Directly get mutable `&mut Rope` is disabled, while `Text` provides all kinds of mutable
  // operations to correctly reset internal cached display width.
  // and hide these details.
  fn rope_mut(&mut self) -> &mut Rope {
    &mut self.rope
  }

  /// Similar with [`Rope::get_line`], but collect and clone a normal string with limited length,
  /// for performance reason when the line is too long to clone.
  fn _clone_line_impl(
    &self,
    line_idx: usize,
    start_char_idx: usize,
    max_chars: usize,
    skip_cache: bool,
  ) -> Option<Rc<String>> {
    self.with_cached_clones(|caches, stats| {
      let key = ClonedLineKey {
        line_idx,
        start_char_idx,
        max_chars,
      };

      if !skip_cache && caches.contains(&key) {
        let result = caches.get(&key).cloned();
        debug_assert!(result.is_some());
        stats.miss_one();
        return result;
      }

      match self.rope.get_line(line_idx) {
        Some(bufline) => match bufline.get_chars_at(start_char_idx) {
          Some(chars_iter) => {
            let mut builder = String::with_capacity(max_chars);
            for (i, c) in chars_iter.enumerate() {
              if i >= max_chars {
                if skip_cache {
                  return Some(Rc::new(builder));
                } else {
                  return Some(
                    self
                      .cached_clones_upsert(caches, stats, &key, || {
                        Rc::new(builder)
                      })
                      .clone(),
                  );
                }
              }
              builder.push(c);
            }

            if skip_cache {
              Some(Rc::new(builder))
            } else {
              Some(
                self
                  .cached_clones_upsert(caches, stats, &key, || {
                    Rc::new(builder)
                  })
                  .clone(),
              )
            }
          }
          None => None,
        },
        None => None,
      }
    })
  }

  /// Similar with [`Rope::get_line`], but collect and clone a normal string with limited length,
  /// for performance reason when the line is too long to clone.
  pub fn clone_line(
    &self,
    line_idx: usize,
    start_char_idx: usize,
    max_chars: usize,
  ) -> Option<Rc<String>> {
    let result1 =
      self._clone_line_impl(line_idx, start_char_idx, max_chars, false);

    // Ensure cached version and non-cached version have same results.
    if cfg!(debug_assertions) {
      let result2 =
        self._clone_line_impl(line_idx, start_char_idx, max_chars, true);
      debug_assert_eq!(result1, result2);
    }
    result1
  }

  // NOTE: Actually here we use a specified algorithm that keeps compatible with the `ropey`
  // library since we heavily rely on it, and cannot do anything without it. But anyway it works
  // great, so let's keep it.
  fn _is_eol_on_line(&self, line: &RopeSlice, char_idx: usize) -> bool {
    let len_chars = line.len_chars();

    // The eol detection logic (NOTE: We don't check the file format option):
    //
    // 1. If the last two chars are CRLF (`\r\n`), and the `char_idx` is one of them, then it is
    //    (one of) the eol. Usually for Windows/Dos.
    // 2. If the last char is CR (`\r`) or LF (`\n`), and `char_idx` is it, then it is the eol.

    let is_crlf = len_chars >= 2
      && char_idx >= len_chars - 2
      && char_idx < len_chars
      && format!("{}{}", line.char(len_chars - 2), line.char(len_chars - 1))
        == EndOfLineOption::Crlf.to_compact_string();
    let is_cr_or_lf = len_chars >= 1
      && char_idx == len_chars - 1
      && (format!("{}", line.char(len_chars - 1))
        == EndOfLineOption::Cr.to_compact_string()
        || format!("{}", line.char(len_chars - 1))
          == EndOfLineOption::Lf.to_compact_string());

    is_crlf || is_cr_or_lf
  }

  // Same logic with `_is_eol_on_line`, except the `char_idx` is absolute on whole rope.
  fn _is_eol_on_whole_text(&self, char_idx: usize) -> bool {
    let r = &self.rope;
    let len_chars = r.len_chars();

    let is_crlf = len_chars >= 2
      && char_idx >= len_chars - 2
      && char_idx < len_chars
      && format!("{}{}", r.char(len_chars - 2), r.char(len_chars - 1))
        == EndOfLineOption::Crlf.to_compact_string();
    let is_cr_or_lf = len_chars >= 1
      && char_idx == len_chars - 1
      && (format!("{}", r.char(len_chars - 1))
        == EndOfLineOption::Cr.to_compact_string()
        || format!("{}", r.char(len_chars - 1))
          == EndOfLineOption::Lf.to_compact_string());

    is_crlf || is_cr_or_lf
  }

  /// Get last char index on line.
  ///
  /// It returns the char index if exists, returns `None` if line not exists or line is empty.
  pub fn last_char_on_line(&self, line_idx: usize) -> Option<usize> {
    match self.rope.get_line(line_idx) {
      Some(line) => {
        let line_len_chars = line.len_chars();
        if line_len_chars > 0 {
          Some(line_len_chars - 1)
        } else {
          None
        }
      }
      None => None,
    }
  }

  /// Get last visible char index on line, it takes below scenarios into considerations:
  ///
  /// - The `\r\n` is eol.
  /// - The `\n` is eol.
  /// - The `\r` is eol. NOTE: This is a legacy on Mac, which is actually not used in
  ///   today's computer.
  ///
  /// It returns the char index if exists, returns `None` if line not exists or line is
  /// empty/blank.
  pub fn last_char_on_line_no_eol(&self, line_idx: usize) -> Option<usize> {
    match self.rope.get_line(line_idx) {
      Some(line) => match self.last_char_on_line(line_idx) {
        Some(last_char) => {
          let mut c = last_char;
          while c > 0 && self._is_eol_on_line(&line, c) {
            c = c.saturating_sub(1);
          }
          if self._is_eol_on_line(&line, c) {
            None
          } else {
            Some(c)
          }
        }
        None => None,
      },
      None => None,
    }
  }

  /// Whether the `line_idx`/`char_idx` is eol (end-of-line).
  pub fn is_eol(&self, line_idx: usize, char_idx: usize) -> bool {
    match self.rope.get_line(line_idx) {
      Some(line) => self._is_eol_on_line(&line, char_idx),
      None => false,
    }
  }
}
// Rope }

// Options {
impl Text {
  pub fn options(&self) -> &BufferOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: &BufferOptions) {
    self.options = *options;
  }
}
// Options }

// Display Width {
impl Text {
  /// See [`ColumnIndex::width_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_before(&self, line_idx: usize, char_idx: usize) -> usize {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .width_before(&self.options, &rope_line, char_idx)
    })
  }

  /// See [`ColumnIndex::width_until`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn width_until(&self, line_idx: usize, char_idx: usize) -> usize {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .width_until(&self.options, &rope_line, char_idx)
    })
  }

  /// See [`ColumnIndex::char_before`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_before(&self, line_idx: usize, width: usize) -> Option<usize> {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .char_before(&self.options, &rope_line, width)
    })
  }

  /// See [`ColumnIndex::char_at`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_at(&self, line_idx: usize, width: usize) -> Option<usize> {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .char_at(&self.options, &rope_line, width)
    })
  }

  /// See [`ColumnIndex::char_after`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn char_after(&self, line_idx: usize, width: usize) -> Option<usize> {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .char_after(&self.options, &rope_line, width)
    })
  }

  /// See [`ColumnIndex::last_char_until`].
  ///
  /// # Panics
  ///
  /// It panics if the `line_idx` doesn't exist in rope.
  pub fn last_char_until(
    &self,
    line_idx: usize,
    width: usize,
  ) -> Option<usize> {
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .last_char_until(&self.options, &rope_line, width)
    })
  }

  fn _retain_cached_cloned_lines<F>(&self, caches: &mut CachedClones, f: F)
  where
    F: Fn(/* line_idx */ &usize) -> bool,
  {
    let to_be_removed: Vec<ClonedLineKey> = caches
      .iter()
      .filter(|(k, _)| !f(&k.line_idx))
      .map(|(k, _)| *k)
      .collect();
    for cloned_key in to_be_removed.iter() {
      caches.pop(cloned_key);
    }
  }

  fn _retain_cached_lines_width<F>(&self, caches: &mut CachedWidth, f: F)
  where
    F: Fn(/* line_idx */ &usize) -> bool,
  {
    let to_be_removed: Vec<usize> = caches
      .iter()
      .filter(|(line_idx, _)| !f(line_idx))
      .map(|(line_idx, _)| *line_idx)
      .collect();
    for cloned_key in to_be_removed.iter() {
      caches.pop(cloned_key);
    }
  }

  /// See [`ColumnIndex::truncate_since_char`].
  fn truncate_cached_line_since_char(&self, line_idx: usize, char_idx: usize) {
    // cached cloned lines
    self.with_cached_clones(|caches, _stats| {
      self._retain_cached_cloned_lines(caches, |line| *line != line_idx);
    });

    // cached lines width
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .truncate_since_char(char_idx)
    })
  }

  #[allow(dead_code)]
  /// See [`ColumnIndex::truncate_since_width`].
  fn truncate_cached_line_since_width(&self, line_idx: usize, width: usize) {
    // cached cloned lines
    self.with_cached_clones(|caches, _stats| {
      self._retain_cached_cloned_lines(caches, |line| *line != line_idx);
    });

    // cached lines width
    self.with_cached_width(|caches, stats| {
      let rope_line = self.rope.line(line_idx);
      self
        .cached_width_upsert(caches, stats, &line_idx, || {
          ColumnIndex::with_capacity(rope_line.len_chars())
        })
        .truncate_since_width(width)
    })
  }

  #[allow(dead_code)]
  /// Remove one cached line.
  fn remove_cached_line(&self, line_idx: usize) {
    // cached cloned lines
    self.with_cached_clones(|caches, _stats| {
      self._retain_cached_cloned_lines(caches, |line| *line != line_idx);
    });

    // cached lines width
    self.with_cached_width(|caches, _stats| {
      caches.pop(&line_idx);
    })
  }

  /// Retain multiple cached lines by lambda function `f`.
  fn retain_cached_lines<F>(&self, f: F)
  where
    F: Fn(/* line_idx */ &usize) -> bool,
  {
    // cached clone lines
    self.with_cached_clones(|caches, _stats| {
      self._retain_cached_cloned_lines(caches, |line| f(line));
    });

    // cached lines width
    self.with_cached_width(|caches, _stats| {
      self._retain_cached_lines_width(caches, |line| f(line));
    });
  }

  /// Clear cache.
  fn clear_cached_lines(&self) {
    self.with_cached_clones(|caches, _stats| {
      caches.clear();
    });
    self.with_cached_width(|caches, _stats| {
      caches.clear();
    });
  }

  #[allow(dead_code)]
  /// Resize cache.
  fn resize_cached_lines(&self, canvas_size: U16Size) {
    let new_cache_size = _cached_size(canvas_size);

    // cached clone lines
    self.with_cached_clones(|caches, _stats| {
      if new_cache_size > caches.cap() {
        caches.resize(new_cache_size);
      }
    });
    self.with_cached_width(|caches, _stats| {
      if new_cache_size > caches.cap() {
        caches.resize(new_cache_size);
      }
    });
  }
}
// Display Width }

#[cfg(test)]
fn _ropeline_to_string(bufline: &ropey::RopeSlice) -> String {
  let mut builder = String::with_capacity(bufline.len_chars());
  for c in bufline.chars() {
    builder.push(c);
  }
  builder
}

impl Text {
  #[cfg(not(test))]
  fn dbg_print_textline_absolutely(
    &mut self,
    _line_idx: usize,
    _absolute_char_idx: usize,
    _msg: &str,
  ) {
  }

  #[cfg(test)]
  fn dbg_print_textline_absolutely(
    &mut self,
    line_idx: usize,
    absolute_char_idx: usize,
    msg: &str,
  ) {
    trace!(
      "{} text line:{},absolute_char:{}",
      msg, line_idx, absolute_char_idx
    );

    match self.rope().get_line(line_idx) {
      Some(line) => {
        trace!("len_chars:{}", line.len_chars());
        let start_char_on_line = self.rope().line_to_char(line_idx);

        let mut builder1 = String::new();
        let mut builder2 = String::new();
        for (i, c) in line.chars().enumerate() {
          let w = self.char_width(c);
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
    for i in 0..self.rope().len_lines() {
      trace!("{i}:{:?}", _ropeline_to_string(&self.rope().line(i)));
    }
  }

  #[cfg(not(test))]
  fn dbg_print_textline(
    &mut self,
    _line_idx: usize,
    _char_idx: usize,
    _msg: &str,
  ) {
  }

  #[cfg(test)]
  fn dbg_print_textline(
    &mut self,
    line_idx: usize,
    char_idx: usize,
    msg: &str,
  ) {
    trace!("{} text line:{},char:{}", msg, line_idx, char_idx);

    match self.rope().get_line(line_idx) {
      Some(bufline) => {
        trace!("len_chars:{}", bufline.len_chars());
        let mut builder1 = String::new();
        let mut builder2 = String::new();
        for (i, c) in bufline.chars().enumerate() {
          let w = self.char_width(c);
          if w > 0 {
            builder1.push(c);
          }
          let s: String =
            std::iter::repeat_n(if i == char_idx { '^' } else { ' ' }, w)
              .collect();
          builder2.push_str(s.as_str());
        }
        trace!("-{}-", builder1);
        trace!("-{}-", builder2);
      }
      None => trace!("line not exist"),
    }

    trace!("{}, whole buffer:", msg);
    for i in 0..self.rope().len_lines() {
      trace!("{i}:{:?}", _ropeline_to_string(&self.rope().line(i)));
    }
  }
}

// Edit {
impl Text {
  /// For text, the editor have to always keep an eol (end-of-line) at the end of text file. It
  /// helps the cursor motion.
  fn append_eol_at_end_if_not_exist(&mut self) {
    let eol = self.options().end_of_line();

    let buffer_len_chars = self.rope.len_chars();
    let last_char_on_buf = buffer_len_chars.saturating_sub(1);
    match self.rope.get_char(last_char_on_buf) {
      Some(_c) => {
        let c_is_eol = self._is_eol_on_whole_text(last_char_on_buf);
        // Only append eol when the whole text rope doesn't have it at end.
        if !c_is_eol {
          self
            .rope_mut()
            .insert(buffer_len_chars, eol.to_compact_string().as_str());
          let inserted_line_idx = self.rope.char_to_line(buffer_len_chars);
          self.retain_cached_lines(|line_idx| *line_idx < inserted_line_idx);
          self.dbg_print_textline_absolutely(
            inserted_line_idx,
            buffer_len_chars,
            "Eol appended(non-empty)",
          );
        }
      }
      None => {
        self
          .rope_mut()
          .insert(0_usize, eol.to_compact_string().as_str());
        self.clear_cached_lines();
        self.dbg_print_textline_absolutely(
          0_usize,
          buffer_len_chars,
          "Eol appended(empty)",
        );
      }
    }
  }

  /// Insert text payload at position `line_idx`/`char_idx`, insert nothing if text payload is
  /// empty.
  ///
  /// # Returns
  /// It returns the new position `(line_idx,char_idx)` after text inserted, it returns `None` if
  /// the text payload is empty.
  ///
  /// # Panics
  /// If the position doesn't exist, or the text payload is empty.
  pub fn insert_at(
    &mut self,
    line_idx: usize,
    char_idx: usize,
    payload: CompactString,
  ) -> (usize, usize) {
    // debug_assert!(!payload.is_empty());
    debug_assert!(self.rope.get_line(line_idx).is_some());
    debug_assert!(char_idx <= self.rope.line(line_idx).len_chars());

    let absolute_line_idx = self.rope.line_to_char(line_idx);
    let absolute_char_idx_before_insert = absolute_line_idx + char_idx;

    self.dbg_print_textline(line_idx, char_idx, "Before insert");

    self
      .rope_mut()
      .insert(absolute_char_idx_before_insert, payload.as_str());

    // The `text` may contains line break '\n', which can interrupts the `line_idx` and we need to
    // re-calculate it.
    let absolute_char_idx_after_inserted =
      absolute_char_idx_before_insert + payload.chars().count();
    let line_idx_after_inserted =
      self.rope.char_to_line(absolute_char_idx_after_inserted);
    let absolute_line_idx_after_inserted =
      self.rope.line_to_char(line_idx_after_inserted);
    let char_idx_after_inserted =
      absolute_char_idx_after_inserted - absolute_line_idx_after_inserted;

    if line_idx == line_idx_after_inserted {
      // If before/after insert, the cursor line doesn't change, it means the inserted text doesn't contain line break, i.e. it is still the same line.
      // Thus only need to truncate chars after insert position on the same line.
      debug_assert!(char_idx_after_inserted >= char_idx);
      let min_cursor_char_idx =
        std::cmp::min(char_idx_after_inserted, char_idx);
      self.truncate_cached_line_since_char(
        line_idx,
        min_cursor_char_idx.saturating_sub(1),
      );
    } else {
      // Otherwise the inserted text contains line breaks, and we have to truncate all the cached lines below the cursor line, because we have new lines.
      let min_cursor_line_idx =
        std::cmp::min(line_idx_after_inserted, line_idx);
      self.retain_cached_lines(|line_idx| *line_idx < min_cursor_line_idx);
    }

    // Append eol at file end if it doesn't exist.
    self.append_eol_at_end_if_not_exist();

    self.dbg_print_textline(
      line_idx_after_inserted,
      char_idx_after_inserted,
      "After inserted",
    );

    (line_idx_after_inserted, char_idx_after_inserted)
  }

  fn _n_chars_to_left(&self, absolute_char_idx: usize, n: usize) -> usize {
    debug_assert!(n > 0);
    let mut i = absolute_char_idx as isize;
    let mut acc = 0;

    while acc < n && i >= 0 {
      let c1 = self.rope.get_char(i as usize);
      let c2 = if i > 0 {
        self.rope.get_char((i - 1) as usize)
      } else {
        None
      };
      if c1.is_some()
        && c2.is_some()
        && format!("{}{}", c2.unwrap(), c1.unwrap())
          == EndOfLineOption::Crlf.to_compact_string()
      {
        i -= 2;
      } else {
        i -= 1;
      }
      acc += 1;
    }
    std::cmp::max(i, 0) as usize
  }

  fn _n_chars_to_right(&self, absolute_char_idx: usize, n: usize) -> usize {
    debug_assert!(n > 0);

    let len_chars = self.rope.len_chars();
    let mut i = absolute_char_idx;
    let mut acc = 0;

    while acc < n && i <= len_chars {
      let c1 = self.rope.get_char(i);
      let c2 = self.rope.get_char(i + 1);
      if c1.is_some()
        && c2.is_some()
        && format!("{}{}", c1.unwrap(), c2.unwrap())
          == EndOfLineOption::Crlf.to_compact_string()
      {
        i += 2;
      } else {
        i += 1;
      }
      acc += 1;
    }
    i
  }

  /// Delete `n` text chars at position `line_idx`/`char_idx`, to either left or right direction.
  ///
  /// 1. If `n<0`, delete to the left direction, i.e. delete the range `[char_idx-n, char_idx)`.
  /// 2. If `n>0`, delete to the right direction, i.e. delete the range `[char_idx, char_idx+n)`.
  /// 3. If `n=0`, delete nothing.
  ///
  /// # Returns
  /// It returns the new position `(line_idx,char_idx)` after deleted, it returns `None` if delete
  /// nothing.
  ///
  /// # Panics
  /// It panics if the position doesn't exist.
  pub fn delete_at(
    &mut self,
    line_idx: usize,
    char_idx: usize,
    n: isize,
  ) -> Option<(usize, usize)> {
    debug_assert!(self.rope.get_line(line_idx).is_some());
    debug_assert!(char_idx < self.rope.line(line_idx).len_chars());

    let cursor_char_absolute_pos_before_delete =
      self.rope.line_to_char(line_idx) + char_idx;

    self.dbg_print_textline(line_idx, char_idx, "Before delete");

    // NOTE: We also need to handle the windows-style line break `\r\n`, i.e. we treat `\r\n` as 1 single char when deleting it.
    let to_be_deleted_range = if n > 0 {
      // Delete to right side, on range `[cursor..cursor+n)`.
      let upper = self
        ._n_chars_to_right(cursor_char_absolute_pos_before_delete, n as usize);
      debug_assert!(upper <= self.rope.len_chars());
      cursor_char_absolute_pos_before_delete..upper
    } else {
      // Delete to left side, on range `[cursor-n,cursor)`.
      let lower = self._n_chars_to_left(
        cursor_char_absolute_pos_before_delete,
        (-n) as usize,
      );
      lower..cursor_char_absolute_pos_before_delete
    };

    if to_be_deleted_range.is_empty() {
      return None;
    }

    self.rope_mut().remove(to_be_deleted_range.clone());

    let cursor_char_absolute_pos_after_deleted = to_be_deleted_range.start;

    let cursor_char_absolute_pos_after_deleted = std::cmp::min(
      cursor_char_absolute_pos_after_deleted,
      self.rope.len_chars(),
    );
    let cursor_line_idx_after_deleted = self
      .rope
      .char_to_line(cursor_char_absolute_pos_after_deleted);
    let cursor_line_absolute_pos_after_deleted =
      self.rope.line_to_char(cursor_line_idx_after_deleted);
    let cursor_char_idx_after_deleted = cursor_char_absolute_pos_after_deleted
      - cursor_line_absolute_pos_after_deleted;

    if line_idx == cursor_line_idx_after_deleted {
      // If before/after insert, the cursor line doesn't change, it means the inserted text doesn't contain line break, i.e. it is still the same line.
      // Thus only need to truncate chars after insert position on the same line.
      let min_cursor_char_idx =
        std::cmp::min(cursor_char_idx_after_deleted, char_idx);
      self.truncate_cached_line_since_char(line_idx, min_cursor_char_idx);
    } else {
      // Otherwise the inserted text contains line breaks, and we have to truncate all the cached lines below the cursor line, because we have new lines.
      let min_cursor_line_idx =
        std::cmp::min(cursor_line_idx_after_deleted, line_idx);
      self.retain_cached_lines(|line_idx| *line_idx < min_cursor_line_idx);
    }

    // Append eol at file end if it doesn't exist.
    self.append_eol_at_end_if_not_exist();

    self.dbg_print_textline(
      cursor_line_idx_after_deleted,
      cursor_char_idx_after_deleted,
      "After deleted",
    );

    Some((cursor_line_idx_after_deleted, cursor_char_idx_after_deleted))
  }

  /// Clear all text payload in current content.
  pub fn clear(&mut self) {
    self.rope_mut().remove(0..);
    self.clear_cached_lines();
  }
}
// Edit }
