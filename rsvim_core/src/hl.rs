//! Highlight and ColorScheme.
//!
//! ColorScheme is defined by a toml config file, which references the theme
//! configuration of the [helix](https://github.com/helix-editor/helix) editor.
//!
//! A colorscheme config file has 3 sections: scope, ui and palette:
//!
//! ```toml
//! [scope]
//! attribute = "white"
//! boolean = { fg = "yellow", bold = true }
//! comment = { fg = "#c0c0c0", bg = "#000000", bold = true, italic = true, underlined = true }
//! keyword = { fg = "#ffffff", bg = "green", italic = true }
//!
//! [scope.source.ruby]
//! attribute = "red"
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
//! `scope` section defines syntax highlightings for programming languages, the
//! value of a scope item can have two formats:
//!
//! - A string defines the foreground text color for that syntax highlighting,
//!   it accepts either ANSI color name, such as "white", "yellow", etc. Or RGB
//!   color code, such as "#ffffff", "#ffff00", etc.
//! - A toml table with below optional attributes:
//!   - `fg`: a string value indicates the foreground text color (ANSI/RGB), it
//!     uses `ui.foreground` if been omitted.
//!   - `bg`: a string value indicates the background text color (ANSI/RGB), it
//!     uses `ui.background` if been omitted.
//!   - `bold`: a boolean value indicates whether the text is bold, by default
//!     it is `false`.
//!   - `italic`: a boolean value indicates whether text is italic, by default
//!     it is `false`.
//!   - `underlined`: a boolean value indicates whether text is underlined, by
//!     default it is `false`.
//!
//! You can overwrite highlightings for specific languages by adding a
//! `[scope.source.{lang}]` section. The `{lang}` should match the grammar name
//! inside a `tree-sitter.json` grammar config.
//!
//! For example in
//! [tree-sitter-ruby](https://github.com/tree-sitter/tree-sitter-ruby),
//! [`tree-sitter.json`](https://github.com/tree-sitter/tree-sitter-ruby/blob/master/tree-sitter.json)
//! grammar name is `"ruby"`:
//!
//! ```json
//! {
//!   "grammars": [
//!     {
//!       "name": "ruby",
//!       "camelcase": "Ruby",
//!       "scope": "source.ruby",
//!       "path": ".",
//!       "file-types": [
//!         "rb"
//!       ],
//!       "highlights": "queries/highlights.scm",
//!       "tags": "queries/tags.scm",
//!       "injection-regex": "ruby"
//!     }
//!   ],
//!   ...
//! }
//! ```
//!
//! In this case, you need to add `[scope.source.ruby]` section for ruby.
//!
//! `ui` section defines other UI highlightings such as common foreground and
//! background text colors. There're some default configs:
//!
//! - `ui.foreground`: uses `white` by default.
//! - `ui.background`: uses `black` by default.
//!
//! `palette` section is a helper section for defining `scope` and `ui` section
//! more easily. By adding a `key=value` pair in palette section, you can use
//! the `key` as a color name in `scope` and `ui` section, syntax highlighting
//! parser will lookup for the real color `value` behind the `key` when loading
//! the colorscheme config.

use crate::prelude::*;
use compact_str::CompactString;
use compact_str::ToCompactString;
use crossterm::style::Attribute;
use crossterm::style::Attributes;
use crossterm::style::Color;
use once_cell::sync::Lazy;

pub const DEFAULT: &str = "default";

// "ui."
pub const FOREGROUND: &str = "foreground";
pub const BACKGROUND: &str = "background";

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

pub static UI_NAMES: Lazy<FoldSet<&'static str>> =
  Lazy::new(|| [FOREGROUND, BACKGROUND].into_iter().collect());

pub static SCOPE_NAMES: Lazy<FoldSet<&'static str>> = Lazy::new(|| {
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
  .collect()
});

#[derive(Debug, Clone, PartialEq, Eq)]
/// Highlight style, including colors and attributes.
pub struct Highlight {
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

  // Colors
  colors: FoldMap<CompactString, Color>,

  // Highlights
  highlights: FoldMap<CompactString, Highlight>,
}

fn parse_code(s: &str, prefix: &str, key: &str) -> TheResult<Color> {
  let parse_hex = |x| {
    u8::from_str_radix(x, 16).map_err(|e| {
      TheErr::LoadColorSchemeFailed(
        format!("{}{}: {:?}", prefix, key, e).to_compact_string(),
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
    Color::try_from(s).map_err(|e| {
      TheErr::LoadColorSchemeFailed(
        format!("{}{}: {:?}", prefix, key, e).to_compact_string(),
      )
    })
  }
}

fn parse_palette(
  colorscheme: &toml::Table,
) -> TheResult<FoldMap<CompactString, Color>> {
  let the_err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("palette.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(palette) = colorscheme.get("palette")
    && let Some(palette_table) = palette.as_table()
  {
    for (k, v) in palette_table.iter() {
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

fn parse_colors(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Color>> {
  let failure = |k: &str| {
    Err(TheErr::LoadColorSchemeFailed(
      format!("ui.{}", k).to_compact_string(),
    ))
  };

  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(ui) = colorscheme.get("ui")
    && let Some(ui_table) = ui.as_table()
  {
    for (key, val) in ui_table.iter() {
      if !UI_NAMES.contains(key.as_str()) {
        return failure(key);
      }
      if val.is_str() {
        let value = val.as_str().unwrap();
        let value = match palette.get(value) {
          Some(code) => *code,
          None => parse_code(value, "ui.", key)?,
        };
        let id = format!("ui.{}", key).to_compact_string();
        result.insert(id, value);
      } else {
        return failure(key);
      }
    }
  }

  Ok(result)
}

fn parse_highlight_as_table(
  key: &str,
  value: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
  colors: &FoldMap<CompactString, Color>,
) -> TheResult<(CompactString, Highlight)> {
  let id = format!("scope.{}", key).to_compact_string();

  let failure = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("scope.{}", k).to_compact_string())
  };

  let parse_color = |x, fallback| -> TheResult<Option<Color>> {
    match value.get(x) {
      Some(x) => {
        let x = x.as_str().ok_or(failure(key))?;
        match palette.get(x) {
          Some(x) => Ok(Some(*x)),
          None => Ok(Some(parse_code(x, "scope.", key)?)),
        }
      }
      None => Ok(colors.get(fallback).copied()),
    }
  };

  let fg = parse_color("fg", "ui.foreground")?;
  let bg = parse_color("bg", "ui.background")?;

  let parse_bool = |x| -> TheResult<bool> {
    match value.get(x) {
      Some(x) => Ok(x.as_bool().ok_or(failure(key))?),
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

  let hl = Highlight {
    id: id.clone(),
    fg,
    bg,
    attr,
  };
  trace!("id:{:?},hl:{:?}", id, hl);
  Ok((id, hl))
}

fn parse_highlight_as_str(
  key: &str,
  value: &str,
  palette: &FoldMap<CompactString, Color>,
  colors: &FoldMap<CompactString, Color>,
) -> TheResult<(CompactString, Highlight)> {
  let id = format!("scope.{}", key).to_compact_string();

  let fg = match palette.get(value) {
    Some(fg) => Some(*fg),
    None => Some(parse_code(value, "scope.", key)?),
  };

  let bg = colors.get("ui.background").copied();
  let attr = Attributes::none();

  let hl = Highlight {
    id: id.clone(),
    fg,
    bg,
    attr,
  };
  trace!("id:{:?},hl:{:?}", id, hl);
  Ok((id, hl))
}

fn parse_highlights(
  colorscheme: &toml::Table,
  colors: &FoldMap<CompactString, Color>,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  let failure = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("scope.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();

  if let Some(scope) = colorscheme.get("scope")
    && let Some(scope_table) = scope.as_table()
  {
    for (key, value) in scope_table.iter() {
      // trace!("key:{:?}, value:{:?}", key, value);
      if key.as_str() == "source" {
        let source_table = value.as_table().unwrap();
        for (lang_key, lang_value) in source_table.iter() {
          // trace!("lang_key:{:?}, lang_value:{:?}", lang_key, lang_value);
          let lang_value_table = lang_value.as_table().unwrap();
          for (attr_key, attr_value) in lang_value_table.iter() {
            let id_key = format!("scope.{}.{}", attr_key, lang_key);
            if !SCOPE_NAMES.contains(attr_key.as_str()) {
              return Err(failure(&id_key));
            }
            if attr_value.is_table() {
              let (id, hl) = parse_highlight_as_table(
                &id_key,
                attr_value.as_table().unwrap(),
                palette,
                colors,
              )?;
              result.insert(id, hl);
            } else if attr_value.is_str() {
              let (id, hl) = parse_highlight_as_str(
                &id_key,
                attr_value.as_str().unwrap(),
                palette,
                colors,
              )?;
              result.insert(id, hl);
            } else {
              return Err(failure(&id_key));
            }
          }
        }
      } else {
        if !SCOPE_NAMES.contains(key.as_str()) {
          return Err(failure(key));
        }
        if value.is_table() {
          let (id, hl) = parse_highlight_as_table(
            key,
            value.as_table().unwrap(),
            palette,
            colors,
          )?;
          result.insert(id, hl);
        } else if value.is_str() {
          let (id, hl) = parse_highlight_as_str(
            key,
            value.as_str().unwrap(),
            palette,
            colors,
          )?;
          result.insert(id, hl);
        } else {
          return Err(failure(key));
        }
      }
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

  pub fn resolve_color(&self, id: &str) -> Option<&Color> {
    debug_assert!(!id.is_empty());
    debug_assert!(!id.trim().is_empty());
    debug_assert_eq!(id.trim(), id);
    debug_assert!(!id.starts_with("."));
    debug_assert!(!id.ends_with("."));

    let mut i = id;
    loop {
      if let Some(color) = self.colors.get(i) {
        return Some(color);
      }
      match i.rfind(".") {
        Some(pos) => {
          i = &i[..pos];
          if i.is_empty() {
            return None;
          }
        }
        None => return None,
      }
    }
  }

  pub fn highlights(&self) -> &FoldMap<CompactString, Highlight> {
    &self.highlights
  }

  pub fn resolve_highlight(&self, id: &str) -> Option<&Highlight> {
    debug_assert!(!id.is_empty());
    debug_assert!(!id.trim().is_empty());
    debug_assert_eq!(id.trim(), id);
    debug_assert!(!id.starts_with("."));
    debug_assert!(!id.ends_with("."));

    let mut i = id;
    loop {
      if let Some(hl) = self.highlights.get(i) {
        return Some(hl);
      }
      match i.rfind(".") {
        Some(pos) => {
          i = &i[..pos];
          if i.is_empty() {
            return None;
          }
        }
        None => return None,
      }
    }
  }
}

#[derive(Debug)]
pub struct ColorSchemeManager {
  // Maps colorscheme name => colorscheme
  colors: FoldMap<CompactString, ColorScheme>,
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

pub static DEFAULT_COLORSCHEME: Lazy<ColorScheme> = Lazy::new(|| {
  let config = toml::toml! {
    [scope]
    boolean = "magenta"
    comment = "cyan"
    constant = "magenta"
    constructor = "cyan"
    embedded = "cyan"
    error = "red"
    function = "green"
    keyword = "yellow"
    markup = "yellow"
    module = "magenta"
    number = "magenta"
    operator = "yellow"
    string = "magenta"
    tag = "magenta"
    type = "green"
    variable = "cyan"
  };
  ColorScheme::from_toml("default", config).unwrap()
});

impl ColorSchemeManager {
  pub fn new() -> Self {
    let mut colors = FoldMap::new();
    colors.insert("default".to_compact_string(), DEFAULT_COLORSCHEME.clone());
    Self { colors }
  }

  pub fn is_empty(&self) -> bool {
    self.colors.is_empty()
  }

  pub fn len(&self) -> usize {
    self.colors.len()
  }

  pub fn get(&self, id: &str) -> Option<&ColorScheme> {
    self.colors.get(id)
  }

  pub fn contains_key(&self, id: &str) -> bool {
    self.colors.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: ColorScheme,
  ) -> Option<ColorScheme> {
    self.colors.insert(key, value)
  }

  pub fn remove(&mut self, id: &str) -> Option<ColorScheme> {
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
