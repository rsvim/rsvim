#![allow(dead_code, unused_variables)]
//! Highlight and ColorScheme.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;
use std::str::FromStr;

// Group
pub const SYN: &str = "syn";
pub const SYN_DOT: &str = "syn.";
pub const UI: &str = "ui";
pub const UI_DOT: &str = "ui.";
pub const PALETTE: &str = "palette";
pub const PALETTE_DOT: &str = "palette.";

// Color
pub const FG: &str = "fg";
pub const FG_DOT: &str = "fg.";
pub const BG: &str = "bg";
pub const BG_DOT: &str = "bg.";

// Attribute
pub const BOLD: &str = "bold";
pub const ITALIC: &str = "italic";
pub const UNDERLINED: &str = "underlined";

pub static TREE_SITTER_HIGHLIGHT_NAMES: Lazy<FoldSet<CompactString>> =
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
    .map(|i| i.to_compact_string())
    .collect::<FoldSet<CompactString>>()
  });

pub static SYNTAX_HIGHLIGHT_NAMES: Lazy<FoldSet<CompactString>> =
  Lazy::new(|| {
    TREE_SITTER_HIGHLIGHT_NAMES
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
  pub fg: Option<Color>,

  /// Background color.
  pub bg: Option<Color>,

  /// Attributes: underlined, bold, italic, etc.
  pub attr: Attributes,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
  // Name.
  name: CompactString,

  // Maps ID => syntax colors
  syntax: FoldMap<CompactString, Highlight>,

  // Maps ID => UI colors
  ui: FoldMap<CompactString, Highlight>,
}

fn parse_code(prefix: &str, k: &str, s: &str) -> TheResult<Color> {
  let parse_hex = |x| {
    u8::from_str_radix(x, 16).map_err(|e| {
      TheErr::LoadColorSchemeFailed(
        format!("{}{}", prefix, k).to_compact_string(),
      )
    })
  };

  if s.starts_with("#") && s.len() == 7 {
    // Parse hex 6 digits, for example: #ffffff
    let s = &s[1..];
    let r = parse_hex(&s[0..2])?;
    let g = parse_hex(&s[2..4])?;
    let b = parse_hex(&s[4..6])?;
    Ok(Color::Rgb { r, g, b })
  } else if s.starts_with("#") && s.len() == 4 {
    // Parse hex 3 digits, for example: #fff
    let s = &s[1..];
    let r = parse_hex(&s[0..1])?;
    let r = r | (r << 4);
    let g = parse_hex(&s[1..2])?;
    let g = g | (g << 4);
    let b = parse_hex(&s[2..3])?;
    let b = b | (b << 4);
    Ok(Color::Rgb { r, g, b })
  } else {
    // Try parse color name
    Color::from_str(s).map_err(|e| {
      TheErr::LoadColorSchemeFailed(
        format!("{}{}", prefix, k).to_compact_string(),
      )
    })
  }
}

fn parse_palette(
  colorscheme: &toml::Table,
) -> TheResult<FoldMap<CompactString, Color>> {
  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(palette_value) = colorscheme.get("palette")
    && let Some(palette) = palette_value.as_table()
  {
    for (k, v) in palette.iter() {
      match v.as_str() {
        Some(val) => {
          let code = parse_code(PALETTE_DOT, k, val)?;
          result.insert(k.as_str().to_compact_string(), code);
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

fn parse_hl(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
  group: &str,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  debug_assert!(group == SYN || group == UI);
  let group_dots: FoldMap<&str, &str> =
    vec![(SYN, SYN_DOT), (UI, UI_DOT)].into_iter().collect();
  let dot = group_dots[group];

  let the_err = |k| {
    TheErr::LoadColorSchemeFailed(format!("{}{}", dot, k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();
  if let Some(group_value) = colorscheme.get(group)
    && let Some(group_table) = group_value.as_table()
  {
    for (key, val) in group_table.iter() {
      let id = format!("{}{}", dot, key.as_str()).to_compact_string();
      if val.is_table() {
        let hl_table = val.as_table().unwrap();

        let parse_color = |x| -> TheResult<Option<Color>> {
          match hl_table.get(x) {
            Some(x) => {
              let x = x.as_str().ok_or(the_err(key))?;
              match palette.get(x) {
                Some(x) => Ok(Some(x.clone())),
                None => Ok(Some(parse_code(dot, key, x)?)),
              }
            }
            None => Ok(None),
          }
        };

        let fg = parse_color(FG)?;
        let bg = parse_color(BG)?;

        let parse_bool = |x| -> TheResult<bool> {
          match hl_table.get(x) {
            Some(x) => {
              let x = x.as_bool().ok_or(the_err(key))?;
              Ok(x)
            }
            None => Ok(false),
          }
        };

        let bold = parse_bool(BOLD)?;
        let italic = parse_bool(ITALIC)?;
        let underlined = parse_bool(UNDERLINED)?;

        let mut attr = Attributes::none();
        if bold {
          attr.set(Attribute::Bold);
        }
        if italic {
          attr.set(Attribute::Italic);
        }
        if underlined {
          attr.set(Attribute::Underlined);
        }

        result.insert(id.clone(), Highlight { id, fg, bg, attr });
      } else if val.is_str() {
        let fg = val.as_str().unwrap();
        let fg = match palette.get(fg) {
          Some(fg) => Some(fg.clone()),
          None => Some(parse_code(dot, key, fg)?),
        };

        let bg = None;
        let attr = Attributes::none();

        result.insert(id.clone(), Highlight { id, fg, bg, attr });
      } else {
        return Err(the_err(key));
      }
    }
  }
  Ok(result)
}

impl ColorScheme {
  pub fn from_empty(name: CompactString) -> Self {
    Self {
      name,
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
  pub fn from_toml(
    name: CompactString,
    colorscheme: toml::Table,
  ) -> TheResult<Self> {
    let palette = parse_palette(&colorscheme)?;
    let syntax = parse_hl(&colorscheme, &palette, SYN)?;
    let ui = parse_hl(&colorscheme, &palette, UI)?;
    Ok(Self { name, syntax, ui })
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn syntax(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.syntax.keys() {
        debug_assert!(k.starts_with(SYN_DOT));
      }
    }
    &self.syntax
  }

  pub fn ui(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.ui.keys() {
        debug_assert!(k.starts_with(UI_DOT));
      }
    }
    &self.ui
  }

  pub fn get(&self, id: &str) -> Option<&Highlight> {
    if id.starts_with(SYN_DOT) {
      self.syntax.get(id)
    } else if id.starts_with(UI_DOT) {
      self.ui.get(id)
    } else {
      None
    }
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
