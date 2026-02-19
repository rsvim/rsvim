#![allow(dead_code, unused_variables)]
//! Highlight and ColorScheme.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;
use std::num::ParseIntError;

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
    .map(|i| i.to_compact_string())
    .collect::<FoldSet<CompactString>>()
  });

#[derive(Debug, Clone, PartialEq, Eq)]
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

  // Map ID => plain colors
  plain: FoldMap<CompactString, Color>,
}

fn parse_code(s: &str, prefix: &str, key: &str) -> TheResult<Color> {
  let parse_hex = |x| {
    u8::from_str_radix(x, 16).map_err(|e| {
      TheErr::LoadColorSchemeFailed(
        format!("{}{}", prefix, key).to_compact_string(),
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
    Err(TheErr::LoadColorSchemeFailed(
      format!("{}{}", prefix, key).to_compact_string(),
    ))
  }
}

fn parse_palette(
  colorscheme: &toml::Table,
) -> TheResult<FoldMap<CompactString, Color>> {
  let the_err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("palette.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(palette_value) = colorscheme.get("palette")
    && let Some(palette) = palette_value.as_table()
  {
    for (k, v) in palette.iter() {
      match v.as_str() {
        Some(val) => {
          let code = parse_code(val, "palette.", k)?;
          result.insert(k.as_str().to_compact_string(), code);
        }
        None => {
          return Err(the_err(k));
        }
      }
    }
  }
  Ok(result)
}

fn plain_keys() -> (
  /* foreground */ &'static str,
  /* background */ &'static str,
  /* ui_foreground */ &'static str,
  /* ui_background */ &'static str,
  /* plains */ FoldSet<&'static str>,
) {
  let foreground = "foreground";
  let background = "background";
  let ui_foreground = "ui.foreground";
  let ui_background = "ui.background";
  let plains = [foreground, background]
    .into_iter()
    .collect::<FoldSet<&'static str>>();
  (foreground, background, ui_foreground, ui_background, plains)
}

fn parse_plain(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Color>> {
  let (foreground, background, ui_foreground, ui_background, plains) =
    plain_keys();

  let the_err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("ui.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(ui_value) = colorscheme.get("ui")
    && let Some(ui_table) = ui_value.as_table()
  {
    for (key, val) in ui_table.iter() {
      if plains.contains(key.as_str()) {
        let id = format!("ui.{}", key).to_compact_string();
        if val.is_str() {
          let val1 = val.as_str().unwrap();
          let val1 = match palette.get(val1) {
            Some(palette_value) => *palette_value,
            None => parse_code(val1, "ui.", key)?,
          };
          result.insert(id.clone(), val1);
        } else {
          return Err(the_err(key));
        }
      }
    }
  }

  if !result.contains_key(ui_foreground) {
    return Err(the_err(foreground));
  }
  if !result.contains_key(ui_background) {
    return Err(the_err(background));
  }

  Ok(result)
}

fn parse_ui(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  let (foreground, background, ui_foreground, ui_background, plains) =
    plain_keys();

  let the_err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("ui.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();
  if let Some(ui_value) = colorscheme.get("ui")
    && let Some(ui_table) = ui_value.as_table()
  {
    for (key, val) in ui_table.iter() {
      if !plains.contains(key.as_str()) {
        let id = format!("ui.{}", key.as_str()).to_compact_string();
        if val.is_table() {
          if key == foreground || key == background {
            return Err(the_err(key));
          }

          let hl_table = val.as_table().unwrap();

          let parse_color = |x| -> TheResult<Option<Color>> {
            match hl_table.get(x) {
              Some(x) => {
                let x = x.as_str().ok_or(the_err(key))?;
                match palette.get(x) {
                  Some(x) => Ok(Some(*x)),
                  None => Ok(Some(parse_code("ui.", key, x)?)),
                }
              }
              None => Ok(None),
            }
          };

          let fg = parse_color("fg")?;
          let bg = parse_color("bg")?;

          let parse_bool = |x| -> TheResult<bool> {
            match hl_table.get(x) {
              Some(x) => {
                let x = x.as_bool().ok_or(the_err(key))?;
                Ok(x)
              }
              None => Ok(false),
            }
          };

          let bold = parse_bool("bold")?;
          let italic = parse_bool("italic")?;
          let underlined = parse_bool("underlined")?;

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
            Some(fg) => Some(*fg),
            None => Some(parse_code("ui.", key, fg)?),
          };

          let bg = None;
          let attr = Attributes::none();

          result.insert(id.clone(), Highlight { id, fg, bg, attr });
        } else {
          return Err(the_err(key));
        }
      }
    }
  }

  if result.contains_key(ui_foreground) {
    return Err(the_err(foreground));
  }
  if result.contains_key(ui_background) {
    return Err(the_err(background));
  }

  Ok(result)
}

fn parse_syn(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  let the_err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("syn.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();
  if let Some(syn_value) = colorscheme.get("syn")
    && let Some(syn_table) = syn_value.as_table()
  {
    for (key, val) in syn_table.iter() {
      let id = format!("syn.{}", key).to_compact_string();
      if val.is_table() {
        let syn_table = val.as_table().unwrap();
        let parse_color = |x| -> TheResult<Option<Color>> {
          match syn_table.get(x) {
            Some(x) => {
              let x = x.as_str().ok_or(the_err(key))?;
              match palette.get(x) {
                Some(x) => Ok(Some(*x)),
                None => Ok(Some(parse_code(x, "syn.", key)?)),
              }
            }
            None => Ok(None),
          }
        };

        let fg = parse_color("fg")?;
        let bg = parse_color("bg")?;

        let parse_bool = |x| -> TheResult<bool> {
          match syn_table.get(x) {
            Some(x) => {
              let x = x.as_bool().ok_or(the_err(key))?;
              Ok(x)
            }
            None => Ok(false),
          }
        };

        let bold = parse_bool("bold")?;
        let italic = parse_bool("italic")?;
        let underlined = parse_bool("underlined")?;

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
          Some(fg) => Some(*fg),
          None => Some(parse_code(fg, "syn.", key)?),
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
  pub fn from_empty(name: &str) -> Self {
    Self {
      name: name.to_compact_string(),
      plain: FoldMap::new(),
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
  pub fn from_toml(name: &str, colorscheme: toml::Table) -> TheResult<Self> {
    let palette = parse_palette(&colorscheme)?;
    let plain = parse_plain(&colorscheme, &palette)?;
    let ui = parse_ui(&colorscheme, &palette)?;
    let syntax = parse_syn(&colorscheme, &palette, "syn")?;

    Ok(Self {
      name: name.to_compact_string(),
      syntax,
      ui,
    })
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn syntax(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.syntax.keys() {
        debug_assert!(k.starts_with("syn."));
      }
    }
    &self.syntax
  }

  pub fn ui(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.ui.keys() {
        debug_assert!(k.starts_with("ui."));
      }
    }
    &self.ui
  }

  pub fn get_raw(&self, id: &str) -> Option<&Highlight> {
    if id.starts_with("syn.") {
      self.syntax.get(id)
    } else if id.starts_with("ui.") {
      self.ui.get(id)
    } else {
      None
    }
  }

  pub fn get(&self, id: &str) -> Option<Highlight> {
    let ui_foreground = "ui.foreground";
    let ui_background = "ui.background";

    if id.starts_with("syn.") {
      self.syntax.get(id).map(|c| {
        let mut c1 = c.clone();
        if c1.fg.is_none() {
          c1.fg = self.ui.get(ui_foreground).unwrap().fg.clone();
        }
        if c1.bg.is_none() {
          c1.bg = self.ui.get(ui_background).unwrap().bg.clone();
        }
      })
    } else if id.starts_with("ui.") {
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
    let mut highlights = FoldMap::new();
    let default_data =
      include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/hl/default.toml"));
    let default_table = default_data.parse::<toml::Table>().unwrap();
    let default_colorscheme =
      ColorScheme::from_toml("default", default_table).unwrap();
    highlights.insert("default".to_compact_string(), default_colorscheme);
    Self { highlights }
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
