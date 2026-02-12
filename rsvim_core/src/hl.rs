#![allow(dead_code, unused_variables)]
//! Highlight and ColorScheme.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;

pub const SYN: &str = "syn";
pub const SYN_DOT: &str = "syn.";
pub const UI: &str = "ui";
pub const UI_DOT: &str = "ui.";
pub const PALETTE: &str = "palette";
pub const PALETTE_DOT: &str = "palette.";

pub static SYNTAX_HIGHLIGHT_NAMES: Lazy<FoldSet<CompactString>> =
  Lazy::new(|| {
    vec![
      "attribute",
      "boolean",
      "carriage-return",
      "comment",
      "comment.documentation",
      "constant",
      "constant.builtin",
      "constructor",
      "constructor.builtin",
      "embedded",
      "error",
      "escape",
      "function",
      "function.builtin",
      "keyword",
      "markup",
      "markup.bold",
      "markup.heading",
      "markup.italic",
      "markup.link",
      "markup.link.url",
      "markup.list",
      "markup.list.checked",
      "markup.list.numbered",
      "markup.list.unchecked",
      "markup.list.unnumbered",
      "markup.quote",
      "markup.raw",
      "markup.raw.block",
      "markup.raw.inline",
      "markup.strikethrough",
      "module",
      "number",
      "operator",
      "property",
      "property.builtin",
      "punctuation",
      "punctuation.bracket",
      "punctuation.delimiter",
      "punctuation.special",
      "string",
      "string.escape",
      "string.regexp",
      "string.special",
      "string.special.symbol",
      "tag",
      "type",
      "type.builtin",
      "variable",
      "variable.builtin",
      "variable.member",
      "variable.parameter",
    ]
    .iter()
    .map(|i| format!("{}{}", SYN_DOT, i).to_compact_string())
    .collect::<FoldSet<CompactString>>()
  });

#[derive(Debug, Clone)]
/// Highlight style, including colors and attributes.
pub struct Highlight {
  /// Style ID
  pub id: CompactString,

  /// Foreground color.
  pub fg: Color,

  /// Background color.
  pub bg: Color,

  /// Attributes: underline, bold, italic, etc.
  pub attr: Attributes,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
  // Name.
  name: CompactString,

  // Maps color name to RGB value.
  // For example: white => #ffffff, black => #000000
  palette: FoldMap<CompactString, CompactString>,

  // Maps ID => syntax colors
  syntax: FoldMap<CompactString, Highlight>,

  // Maps ID => UI colors
  ui: FoldMap<CompactString, Highlight>,
}

fn parse_palette(
  colorscheme: &toml::Table,
) -> TheResult<FoldMap<CompactString, CompactString>> {
  let mut result: FoldMap<CompactString, CompactString> = FoldMap::new();
  if let Some(palette_value) = colorscheme.get("palette")
    && let Some(palette) = palette_value.as_table()
  {
    for (k, v) in palette.iter() {
      match v.as_str() {
        Some(val) => {
          result
            .insert(k.as_str().to_compact_string(), val.to_compact_string());
        }
        None => {
          return Err(TheErr::LoadColorSchemeFailed(
            format!("{}{}", PALETTE_DOT, k.as_str()).to_compact_string(),
          ));
        }
      }
    }
  }
  Ok(result)
}

fn parse_color_code(code: &str) -> Color {
  if code.starts_with("#") {
    Color::Rgb { r: (), g: (), b: () }
  }
}

fn parse_hl(
  colorscheme: &toml::Table,
  group: &str,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  debug_assert!(group == SYN || group == UI);
  let dot_names: FoldMap<&str, &str> =
    vec![(SYN, SYN_DOT), (UI, UI_DOT)].into_iter().collect();
  let dot = dot_names[group];

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();
  if let Some(palette_value) = colorscheme.get("palette")
    && let Some(palette) = palette_value.as_table()
  {
    for (k, v) in palette.iter() {
      let id = format!("{}{}", dot, k.as_str()).to_compact_string();
      if v.is_table() {
        let v = v.as_table().unwrap();
        result.insert(
          id,
          Highlight {
            id,
            fg: Color::
          },
        );
      } else if v.is_str() {
        let v = v.as_str().unwrap();
      } else {
        return Err(TheErr::LoadColorSchemeFailed(
          format!("{}{}", dot, k.as_str()).to_compact_string(),
        ));
      }
    }
  }
  Ok(result)
}

impl ColorScheme {
  pub fn from_empty(name: CompactString) -> Self {
    Self {
      name,
      palette: FoldMap::new(),
      syntax: FoldMap::new(),
      ui: FoldMap::new(),
    }
  }

  /// A ColorScheme can be defined with a toml file, for example:
  /// ```toml
  /// [syn]
  /// attribute = "white"
  /// boolean = { fg = "yellow", bold = true }
  ///
  /// [ui]
  /// background = "#000000"
  ///
  /// [palette]
  /// white = "#ffffff"
  /// black = "#000000"
  /// yellow = "#ffff00"
  /// ```
  pub fn from_toml(name: CompactString, colorscheme: toml::Table) -> Self {
    let mut syntax: FoldMap<CompactString, Highlight> = FoldMap::new();
    let mut ui: FoldMap<CompactString, Highlight> = FoldMap::new();

    Self {
      name,
      palette,
      syntax,
      ui,
    }
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn palette(&self) -> &FoldMap<CompactString, CompactString> {
    &self.palette
  }

  pub fn palette_mut(&mut self) -> &mut FoldMap<CompactString, CompactString> {
    &mut self.palette
  }

  pub fn syntax(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.syntax.keys() {
        debug_assert!(k.starts_with(SYN_DOT));
      }
    }
    &self.syntax
  }

  pub fn syntax_mut(&mut self) -> &mut FoldMap<CompactString, Highlight> {
    &mut self.syntax
  }

  pub fn ui(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.ui.keys() {
        debug_assert!(k.starts_with(UI_DOT));
      }
    }
    &self.ui
  }

  pub fn ui_mut(&self) -> &FoldMap<CompactString, Highlight> {
    &self.ui
  }
}

#[derive(Debug)]
pub struct ColorSchemeManager {
  // Maps highlight ID => highlight
  highlights: FoldMap<CompactString, ColorScheme>,
}

impl Default for ColorSchemeManager {
  fn default() -> Self {
    Self::new()
  }
}

pub type ColorSchemeManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, ColorScheme>;
pub type ColorSchemeManagerValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, ColorScheme>;
pub type ColorSchemeManagerIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, ColorScheme>;

impl ColorSchemeManager {
  pub fn new() -> Self {
    Self {
      highlights: FoldMap::new(),
    }
  }

  pub fn is_empty(&self) -> bool {
    self.highlights.is_empty()
  }

  pub fn len(&self) -> usize {
    self.highlights.len()
  }

  pub fn get(&self, id: &CompactString) -> Option<&ColorScheme> {
    self.highlights.get(id)
  }

  pub fn contains_key(&self, id: &CompactString) -> bool {
    self.highlights.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: ColorScheme,
  ) -> Option<ColorScheme> {
    self.highlights.insert(key, value)
  }

  pub fn remove(&mut self, id: &CompactString) -> Option<ColorScheme> {
    self.highlights.remove(id)
  }

  pub fn keys(&self) -> ColorSchemeManagerKeys<'_> {
    self.highlights.keys()
  }

  pub fn values(&self) -> ColorSchemeManagerValues<'_> {
    self.highlights.values()
  }

  pub fn iter(&self) -> ColorSchemeManagerIter<'_> {
    self.highlights.iter()
  }
}
