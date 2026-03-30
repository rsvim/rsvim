//! Highlight and ColorScheme.
//!
//! ColorScheme is defined by a toml config file, which references the theme
//! configuration of the [helix](https://github.com/helix-editor/helix) editor.
//!
//! A colorscheme config file has 3 sections: scope, ui and palette:
//!
//! ```toml
//! # scope
//! attribute = "white"
//! boolean = { fg = "yellow", bold = true }
//! comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
//! keyword = { fg = "#ffffff", bg = "green", italic = true }
//!
//! [ui]
//! background = "#000000"
//!
//! [palette]
//! # white = "#ffffff"
//! black = "#000000"
//! yellow = "#ffff00"
//! green = "#00ff00"
//!
//! # Never used
//! grey = "#c0c0c0"
//! ```
//!
//! `scope` section doesn't explicitly defines a `[scope]` section like
//! `[ui]` or `[palette]`, it actually comes from the
//! [tree-sitter highlght names](https://github.com/tree-sitter/tree-sitter/blob/cf302b07d1cae984068b7eb44a6e44529566a8c9/crates/highlight/src/highlight.rs#L30).
//! It defines syntax highlightings for programming languages, the value of a
//! highlight can have two formats:
//!
//! - A string defines the text color for the highlighting, it accepts either
//!   ANSI color name (such as "white", "yellow", etc) or RGB color code (such
//!   as "#ffffff", "#ffff00", etc).
//! - A toml table with below optional attributes:
//!   - `fg`: a string value indicates the foreground text color (ANSI/RGB), it
//!     uses `ui.text` if been omitted.
//!   - `bg`: a string value indicates the background text color (ANSI/RGB), it
//!     uses `ui.background` if been omitted.
//!   - `bold`: a boolean value indicates whether the text is bold, by default
//!     it is `false`.
//!   - `italic`: a boolean value indicates whether text is italic, by default
//!     it is `false`.
//!   - `underlined`: a boolean value indicates whether text is underlined, by
//!     default it is `false`.
//!
//! `ui` section defines other UI highlightings such as common text colors and
//! background colors. There're some default configs:
//!
//! - `ui.text`: uses `white` by default.
//! - `ui.background`: uses `black` by default.
//!
//! `palette` section is a helper section for defining `[scope]` and `[ui]`
//! section more easily. By adding a `key=value` pair in palette section, you
//! can use the `key` as a color name in `[scope]` and `[ui]` section, syntax
//! highlighting parser will lookup for the real color `value` behind the `key`
//! when loading the colorscheme config.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;
use std::sync::LazyLock;

pub const DEFAULT: &str = "default";
pub const RESET_COLOR: Color = Color::Reset;

// "ui."
pub const TEXT: &str = "text";
pub const BACKGROUND: &str = "background";
pub const UI_TEXT: &str = "ui.text";
pub const UI_BACKGROUND: &str = "ui.background";

// "scope."
pub const ATTRIBUTE: &str = "attribute";
pub const BOOLEAN: &str = "boolean";
pub const CARRIAGE_RETURN: &str = "carriage-return";
pub const COMMENT: &str = "comment";
pub const COMMENT_DOCUMENTATION: &str = "comment.documentation";
pub const CONSTANT: &str = "constant";
pub const CONSTANT_BUILTIN: &str = "constant.builtin";
pub const CONSTRUCTOR: &str = "constructor";
pub const CONSTRUCTOR_BUILTIN: &str = "constructor.builtin";
pub const EMBEDDED: &str = "embedded";
pub const ERROR: &str = "error";
pub const ESCAPE: &str = "escape";
pub const FUNCTION: &str = "function";
pub const FUNCTION_BUILTIN: &str = "function.builtin";
pub const KEYWORD: &str = "keyword";
pub const MARKUP: &str = "markup";
pub const MARKUP_BOLD: &str = "markup.bold";
pub const MARKUP_HEADING: &str = "markup.heading";
pub const MARKUP_ITALIC: &str = "markup.italic";
pub const MARKUP_LINK: &str = "markup.link";
pub const MARKUP_LINK_URL: &str = "markup.link.url";
pub const MARKUP_LIST: &str = "markup.list";
pub const MARKUP_LIST_CHECKED: &str = "markup.list.checked";
pub const MARKUP_LIST_NUMBERED: &str = "markup.list.numbered";
pub const MARKUP_LIST_UNCHECKED: &str = "markup.list.unchecked";
pub const MARKUP_LIST_UNNUMBERED: &str = "markup.list.unnumbered";
pub const MARKUP_QUOTE: &str = "markup.quote";
pub const MARKUP_RAW: &str = "markup.raw";
pub const MARKUP_RAW_BLOCK: &str = "markup.raw.block";
pub const MARKUP_RAW_INLINE: &str = "markup.raw.inline";
pub const MARKUP_STRIKETHROUGH: &str = "markup.strikethrough";
pub const MODULE: &str = "module";
pub const NUMBER: &str = "number";
pub const OPERATOR: &str = "operator";
pub const PROPERTY: &str = "property";
pub const PROPERTY_BUILTIN: &str = "property.builtin";
pub const PUNCTUATION: &str = "punctuation";
pub const PUNCTUATION_BRACKET: &str = "punctuation.bracket";
pub const PUNCTUATION_DELIMITER: &str = "punctuation.delimiter";
pub const PUNCTUATION_SPECIAL: &str = "punctuation.special";
pub const STRING: &str = "string";
pub const STRING_ESCAPE: &str = "string.escape";
pub const STRING_REGEXP: &str = "string.regexp";
pub const STRING_SPECIAL: &str = "string.special";
pub const STRING_SPECIAL_SYMBOL: &str = "string.special.symbol";
pub const TAG: &str = "tag";
pub const TYPE: &str = "type";
pub const TYPE_BUILTIN: &str = "type.builtin";
pub const VARIABLE: &str = "variable";
pub const VARIABLE_BUILTIN: &str = "variable.builtin";
pub const VARIABLE_MEMBER: &str = "variable.member";
pub const VARIABLE_PARAMETER: &str = "variable.parameter";

pub static UI_NAMES: LazyLock<FoldMap<&'static str, &'static str>> =
  LazyLock::new(|| [TEXT, BACKGROUND].into_iter().map(|i| (i, i)).collect());

pub static SCOPE_NAMES: LazyLock<FoldMap<&'static str, &'static str>> =
  LazyLock::new(|| {
    [
      ATTRIBUTE,
      BOOLEAN,
      CARRIAGE_RETURN,
      COMMENT,
      COMMENT_DOCUMENTATION,
      CONSTANT,
      CONSTANT_BUILTIN,
      CONSTRUCTOR,
      CONSTRUCTOR_BUILTIN,
      EMBEDDED,
      ERROR,
      ESCAPE,
      FUNCTION,
      FUNCTION_BUILTIN,
      KEYWORD,
      MARKUP,
      MARKUP_BOLD,
      MARKUP_HEADING,
      MARKUP_ITALIC,
      MARKUP_LINK,
      MARKUP_LINK_URL,
      MARKUP_LIST,
      MARKUP_LIST_CHECKED,
      MARKUP_LIST_NUMBERED,
      MARKUP_LIST_UNCHECKED,
      MARKUP_LIST_UNNUMBERED,
      MARKUP_QUOTE,
      MARKUP_RAW,
      MARKUP_RAW_BLOCK,
      MARKUP_RAW_INLINE,
      MARKUP_STRIKETHROUGH,
      MODULE,
      NUMBER,
      OPERATOR,
      PROPERTY,
      PROPERTY_BUILTIN,
      PUNCTUATION,
      PUNCTUATION_BRACKET,
      PUNCTUATION_DELIMITER,
      PUNCTUATION_SPECIAL,
      STRING,
      STRING_ESCAPE,
      STRING_REGEXP,
      STRING_SPECIAL,
      STRING_SPECIAL_SYMBOL,
      TAG,
      TYPE,
      TYPE_BUILTIN,
      VARIABLE,
      VARIABLE_BUILTIN,
      VARIABLE_MEMBER,
      VARIABLE_PARAMETER,
    ]
    .into_iter()
    .map(|i| (i, i))
    .collect()
  });

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Highlight style, including colors and attributes.
pub struct Highlight {
  /// Foreground color.
  pub fg: Option<Color>,

  /// Background color.
  pub bg: Option<Color>,

  /// Attributes: underlined, bold, italic, etc.
  pub attrs: Attributes,
}

#[derive(Debug, Clone)]
pub struct ColorScheme {
  // Name.
  name: CompactString,

  // Colors
  colors: FoldMap<CompactString, Color>,

  // Highlights
  highlights: FoldMap<CompactString, Highlight>,
}

arc_ptr!(ColorScheme);

fn parse_color(s: &str, prefix: &str, key: &str) -> TheResult<Color> {
  let parse_hex = |x| {
    u8::from_str_radix(x, 16).map_err(|_e| {
      TheErr::LoadColorSchemeFailed(
        format!("{prefix}{key}={:?}", s).to_compact_string(),
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
    Color::try_from(s).map_err(|_e| {
      TheErr::LoadColorSchemeFailed(
        format!("{prefix}{key}={:?}", s).to_compact_string(),
      )
    })
  }
}

fn parse_palette(
  colorscheme: &toml::Table,
) -> TheResult<FoldMap<CompactString, Color>> {
  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(palette) = colorscheme.get("palette")
    && let Some(palette_table) = palette.as_table()
  {
    for (key, value) in palette_table.iter() {
      match value.as_str() {
        Some(value) => {
          let code = parse_color(value, "palette.", key)?;
          result.insert(key.as_str().to_compact_string(), code);
        }
        None => {
          return Err(TheErr::LoadColorSchemeFailed(
            format!("palette.{}={:?}", key, value).to_compact_string(),
          ));
        }
      }
    }
  }
  Ok(result)
}

fn parse_colors(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Color>> {
  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(ui) = colorscheme.get("ui")
    && let Some(ui_table) = ui.as_table()
  {
    for (key, value) in ui_table.iter() {
      if !UI_NAMES.contains_key(key.as_str()) {
        return Err(TheErr::LoadColorSchemeFailed(
          format!("ui.{}={:?}", key, value).to_compact_string(),
        ));
      }
      if value.is_str() {
        let value = value.as_str().unwrap();
        let value = match palette.get(value) {
          Some(code) => *code,
          None => parse_color(value, "ui.", key)?,
        };
        let id = format!("ui.{}", key).to_compact_string();
        result.insert(id, value);
      } else {
        return Err(TheErr::LoadColorSchemeFailed(
          format!("ui.{}={:?}", key, value).to_compact_string(),
        ));
      }
    }
  }

  Ok(result)
}

fn parse_hl_as_table(
  key: &str,
  value: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
  colors: &FoldMap<CompactString, Color>,
) -> TheResult<(CompactString, Highlight)> {
  let parse_color = |x, fallback| -> TheResult<Option<Color>> {
    match value.get(x) {
      Some(x_value) => {
        let x_value = x_value.as_str().ok_or(TheErr::LoadColorSchemeFailed(
          format!("{}={:?}", key, x_value).to_compact_string(),
        ))?;
        match palette.get(x_value) {
          Some(x) => Ok(Some(*x)),
          None => Ok(Some(parse_color(x_value, "scope.", key)?)),
        }
      }
      None => Ok(colors.get(fallback).copied()),
    }
  };

  let fg = parse_color("fg", UI_TEXT)?;
  let bg = parse_color("bg", UI_BACKGROUND)?;

  let parse_bool = |x| -> TheResult<bool> {
    match value.get(x) {
      Some(x_value) => {
        Ok(x_value.as_bool().ok_or(TheErr::LoadColorSchemeFailed(
          format!("{}={:?}", key, x_value).to_compact_string(),
        ))?)
      }
      None => Ok(false),
    }
  };

  let bold = parse_bool("bold")?;
  let dim = parse_bool("dim")?;
  let italic = parse_bool("italic")?;
  let underlined = parse_bool("underlined")?;

  let mut attrs = Attributes::none();
  if bold {
    attrs.set(Attribute::Bold);
  }
  if dim {
    attrs.set(Attribute::Dim);
  }
  if italic {
    attrs.set(Attribute::Italic);
  }
  if underlined {
    attrs.set(Attribute::Underlined);
  }

  let hl = Highlight { fg, bg, attrs };
  trace!("id:{:?},hl:{:?}", key, hl);
  Ok((key.to_compact_string(), hl))
}

fn parse_hl_as_str(
  key: &str,
  value: &str,
  palette: &FoldMap<CompactString, Color>,
  colors: &FoldMap<CompactString, Color>,
) -> TheResult<(CompactString, Highlight)> {
  let fg = match palette.get(value) {
    Some(fg) => Some(*fg),
    None => Some(parse_color(value, "scope.", key)?),
  };

  let bg = colors.get(UI_BACKGROUND).copied();
  let attrs = Attributes::none();

  let hl = Highlight { fg, bg, attrs };
  trace!("id:{:?},hl:{:?}", key, hl);
  Ok((key.to_compact_string(), hl))
}

fn parse_highlights(
  colorscheme: &toml::Table,
  colors: &FoldMap<CompactString, Color>,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();

  for (key, value) in colorscheme.iter() {
    if SCOPE_NAMES.contains_key(key.as_str()) {
      if value.is_table() {
        let (id, hl) =
          parse_hl_as_table(key, value.as_table().unwrap(), palette, colors)?;
        result.insert(id, hl);
      } else if value.is_str() {
        let (id, hl) =
          parse_hl_as_str(key, value.as_str().unwrap(), palette, colors)?;
        result.insert(id, hl);
      } else {
        return Err(TheErr::LoadColorSchemeFailed(
          format!("{}={:?}", key, value).to_compact_string(),
        ));
      }
    } else if key.as_str() != "ui" && key.as_str() != "palette" {
      return Err(TheErr::LoadColorSchemeFailed(
        format!("{}={:?}", key, value).to_compact_string(),
      ));
    }
  }

  Ok(result)
}

impl ColorScheme {
  pub fn from_empty(name: &str) -> Self {
    Self {
      name: name.to_compact_string(),
      colors: FoldMap::new(),
      highlights: FoldMap::new(),
    }
  }

  /// A ColorScheme can be defined with a toml file.
  pub fn from_toml(name: &str, colorscheme: toml::Table) -> TheResult<Self> {
    let palette = parse_palette(&colorscheme)?;
    let colors = parse_colors(&colorscheme, &palette)?;
    let highlights = parse_highlights(&colorscheme, &colors, &palette)?;

    Ok(Self {
      name: name.to_compact_string(),
      colors,
      highlights,
    })
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn colors(&self) -> &FoldMap<CompactString, Color> {
    &self.colors
  }

  pub fn highlights(&self) -> &FoldMap<CompactString, Highlight> {
    &self.highlights
  }

  pub fn resolve(&self, color: &Option<Color>, fallback: &str) -> Color {
    color.unwrap_or(self.colors.get(fallback).copied().unwrap_or(Color::Reset))
  }

  pub fn ui_text(&self) -> Color {
    self.colors.get(UI_TEXT).copied().unwrap_or(Color::Reset)
  }

  pub fn ui_background(&self) -> Color {
    self
      .colors
      .get(UI_BACKGROUND)
      .copied()
      .unwrap_or(Color::Reset)
  }

  pub fn resolve_fg(&self, fg: &Option<Color>) -> Color {
    fg.unwrap_or(self.ui_text())
  }

  pub fn resolve_bg(&self, bg: &Option<Color>) -> Color {
    bg.unwrap_or(self.ui_background())
  }
}

#[derive(Debug)]
pub struct ColorSchemeManager {
  // Maps colorscheme name => colorscheme
  colors: FoldMap<CompactString, ColorSchemeArc>,
}

impl Default for ColorSchemeManager {
  fn default() -> Self {
    Self::new()
  }
}

pub type ColorSchemeManagerKeys<'a> =
  std::collections::hash_map::Keys<'a, CompactString, ColorSchemeArc>;
pub type ColorSchemeManagerValues<'a> =
  std::collections::hash_map::Values<'a, CompactString, ColorSchemeArc>;
pub type ColorSchemeManagerIter<'a> =
  std::collections::hash_map::Iter<'a, CompactString, ColorSchemeArc>;

pub fn default_colorscheme() -> ColorSchemeArc {
  let config = toml::toml! {
    boolean = "magenta"
    comment = "cyan"
    constant = "magenta"
    constructor = "cyan"
    embedded = "cyan"
    error = "red"
    function = "cyan"
    keyword = "green"
    markup = "yellow"
    "markup.heading" = "red"
    "markup.link" = "magenta"
    "markup.list" = "yellow"
    "markup.quote" = "yellow"
    "markup.raw" = "yellow"
    module = "magenta"
    number = "magenta"
    operator = "yellow"
    punctuation = "cyan"
    string = "magenta"
    tag = "magenta"
    "type" = "green"
    variable = "cyan"

    [ui]
    text = "white"
    background = "black"
  };
  let cs = ColorScheme::from_toml(DEFAULT, config).unwrap();
  ColorScheme::to_arc(cs)
}

impl ColorSchemeManager {
  pub fn new() -> Self {
    let mut colors = FoldMap::new();
    colors.insert(DEFAULT.to_compact_string(), default_colorscheme());
    Self { colors }
  }

  pub fn is_empty(&self) -> bool {
    self.colors.is_empty()
  }

  pub fn len(&self) -> usize {
    self.colors.len()
  }

  pub fn get(&self, id: &str) -> Option<&ColorSchemeArc> {
    self.colors.get(id)
  }

  pub fn contains_key(&self, id: &str) -> bool {
    self.colors.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: ColorSchemeArc,
  ) -> Option<ColorSchemeArc> {
    self.colors.insert(key, value)
  }

  pub fn remove(&mut self, id: &str) -> Option<ColorSchemeArc> {
    self.colors.remove(id)
  }

  pub fn keys(&self) -> ColorSchemeManagerKeys<'_> {
    self.colors.keys()
  }

  pub fn values(&self) -> ColorSchemeManagerValues<'_> {
    self.colors.values()
  }

  pub fn iter(&self) -> ColorSchemeManagerIter<'_> {
    self.colors.iter()
  }
}
