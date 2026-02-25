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
//! boolean = "red"
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
//! `scope.source.{lang}` subsection. The `source.{lang}` part should match the
//! tree-sitter grammar `grammars.0.scope` field inside the `tree-sitter.json`
//! config.
//!
//! For example in
//! [tree-sitter-ruby](https://github.com/tree-sitter/tree-sitter-ruby), the
//! [`tree-sitter.json`](https://github.com/tree-sitter/tree-sitter-ruby/blob/master/tree-sitter.json)
//! is:
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
//! To overwrite highlightings for ruby in your colorscheme, you need to add a
//! subsection `scope.source.ruby`, the `source.ruby` matches the
//! line `"scope": "source.ruby"` in the `tree-sitter.json` file.
//!
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

// Default UI colors
pub const DEFAULT_FOREGROUND_COLOR: Color = Color::White;
pub const DEFAULT_BACKGROUND_COLOR: Color = Color::Black;

// "ui."
pub const FOREGROUND: &str = "foreground";
pub const BACKGROUND: &str = "background";
pub const UI_FOREGROUND: &str = "ui.foreground";
pub const UI_BACKGROUND: &str = "ui.background";

// "syn."
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
pub const SYN_ATTRIBUTE: &str = "syn.attribute";
pub const SYN_BOOLEAN: &str = "syn.boolean";
pub const SYN_CARRIAGE_RETURN: &str = "syn.carriage-return";
pub const SYN_COMMENT: &str = "syn.comment";
pub const SYN_COMMENT_DOCUMENTATION: &str = "syn.comment.documentation";
pub const SYN_CONSTANT: &str = "syn.constant";
pub const SYN_CONSTANT_BUILTIN: &str = "syn.constant.builtin";
pub const SYN_CONSTRUCTOR: &str = "syn.constructor";
pub const SYN_CONSTRUCTOR_BUILTIN: &str = "syn.constructor.builtin";
pub const SYN_EMBEDDED: &str = "syn.embedded";
pub const SYN_ERROR: &str = "syn.error";
pub const SYN_ESCAPE: &str = "syn.escape";
pub const SYN_FUNCTION: &str = "syn.function";
pub const SYN_FUNCTION_BUILTIN: &str = "syn.function.builtin";
pub const SYN_KEYWORD: &str = "syn.keyword";
pub const SYN_MARKUP: &str = "syn.markup";
pub const SYN_MARKUP_BOLD: &str = "syn.markup.bold";
pub const SYN_MARKUP_HEADING: &str = "syn.markup.heading";
pub const SYN_MARKUP_ITALIC: &str = "syn.markup.italic";
pub const SYN_MARKUP_LINK: &str = "syn.markup.link";
pub const SYN_MARKUP_LINK_URL: &str = "syn.markup.link.url";
pub const SYN_MARKUP_LIST: &str = "syn.markup.list";
pub const SYN_MARKUP_LIST_CHECKED: &str = "syn.markup.list.checked";
pub const SYN_MARKUP_LIST_NUMBERED: &str = "syn.markup.list.numbered";
pub const SYN_MARKUP_LIST_UNCHECKED: &str = "syn.markup.list.unchecked";
pub const SYN_MARKUP_LIST_UNNUMBERED: &str = "syn.markup.list.unnumbered";
pub const SYN_MARKUP_QUOTE: &str = "syn.markup.quote";
pub const SYN_MARKUP_RAW: &str = "syn.markup.raw";
pub const SYN_MARKUP_RAW_BLOCK: &str = "syn.markup.raw.block";
pub const SYN_MARKUP_RAW_INLINE: &str = "syn.markup.raw.inline";
pub const SYN_MARKUP_STRIKETHROUGH: &str = "syn.markup.strikethrough";
pub const SYN_MODULE: &str = "syn.module";
pub const SYN_NUMBER: &str = "syn.number";
pub const SYN_OPERATOR: &str = "syn.operator";
pub const SYN_PROPERTY: &str = "syn.property";
pub const SYN_PROPERTY_BUILTIN: &str = "syn.property.builtin";
pub const SYN_PUNCTUATION: &str = "syn.punctuation";
pub const SYN_PUNCTUATION_BRACKET: &str = "syn.punctuation.bracket";
pub const SYN_PUNCTUATION_DELIMITER: &str = "syn.punctuation.delimiter";
pub const SYN_PUNCTUATION_SPECIAL: &str = "syn.punctuation.special";
pub const SYN_STRING: &str = "syn.string";
pub const SYN_STRING_ESCAPE: &str = "syn.string.escape";
pub const SYN_STRING_REGEXP: &str = "syn.string.regexp";
pub const SYN_STRING_SPECIAL: &str = "syn.string.special";
pub const SYN_STRING_SPECIAL_SYMBOL: &str = "syn.string.special.symbol";
pub const SYN_TAG: &str = "syn.tag";
pub const SYN_TYPE: &str = "syn.type";
pub const SYN_TYPE_BUILTIN: &str = "syn.type.builtin";
pub const SYN_VARIABLE: &str = "syn.variable";
pub const SYN_VARIABLE_BUILTIN: &str = "syn.variable.builtin";
pub const SYN_VARIABLE_MEMBER: &str = "syn.variable.member";
pub const SYN_VARIABLE_PARAMETER: &str = "syn.variable.parameter";

pub static TREESITTER_HIGHLIGHTS: Lazy<FoldSet<&str>> = Lazy::new(|| {
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
  .collect::<FoldSet<&str>>()
});

pub static SYN_TREESITTER_HIGHLIGHTS: Lazy<FoldSet<&str>> = Lazy::new(|| {
  [
    SYN_ATTRIBUTE,
    SYN_BOOLEAN,
    SYN_CARRIAGE_RETURN,
    SYN_COMMENT,
    SYN_COMMENT_DOCUMENTATION,
    SYN_CONSTANT,
    SYN_CONSTANT_BUILTIN,
    SYN_CONSTRUCTOR,
    SYN_CONSTRUCTOR_BUILTIN,
    SYN_EMBEDDED,
    SYN_ERROR,
    SYN_ESCAPE,
    SYN_FUNCTION,
    SYN_FUNCTION_BUILTIN,
    SYN_KEYWORD,
    SYN_MARKUP,
    SYN_MARKUP_BOLD,
    SYN_MARKUP_HEADING,
    SYN_MARKUP_ITALIC,
    SYN_MARKUP_LINK,
    SYN_MARKUP_LINK_URL,
    SYN_MARKUP_LIST,
    SYN_MARKUP_LIST_CHECKED,
    SYN_MARKUP_LIST_NUMBERED,
    SYN_MARKUP_LIST_UNCHECKED,
    SYN_MARKUP_LIST_UNNUMBERED,
    SYN_MARKUP_QUOTE,
    SYN_MARKUP_RAW,
    SYN_MARKUP_RAW_BLOCK,
    SYN_MARKUP_RAW_INLINE,
    SYN_MARKUP_STRIKETHROUGH,
    SYN_MODULE,
    SYN_NUMBER,
    SYN_OPERATOR,
    SYN_PROPERTY,
    SYN_PROPERTY_BUILTIN,
    SYN_PUNCTUATION,
    SYN_PUNCTUATION_BRACKET,
    SYN_PUNCTUATION_DELIMITER,
    SYN_PUNCTUATION_SPECIAL,
    SYN_STRING,
    SYN_STRING_ESCAPE,
    SYN_STRING_REGEXP,
    SYN_STRING_SPECIAL,
    SYN_STRING_SPECIAL_SYMBOL,
    SYN_TAG,
    SYN_TYPE,
    SYN_TYPE_BUILTIN,
    SYN_VARIABLE,
    SYN_VARIABLE_BUILTIN,
    SYN_VARIABLE_MEMBER,
    SYN_VARIABLE_PARAMETER,
  ]
  .into_iter()
  .collect::<FoldSet<&str>>()
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

  // Plain colors
  foreground: Color,
  background: Color,

  // Syntax colors
  syn: FoldMap<CompactString, Highlight>,
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

fn parse_plain_colors(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Color>> {
  let plain_colors = [FOREGROUND, BACKGROUND]
    .into_iter()
    .collect::<FoldSet<&str>>();

  let err = |k: &str| {
    Err(TheErr::LoadColorSchemeFailed(
      format!("ui.{}", k).to_compact_string(),
    ))
  };

  let mut result: FoldMap<CompactString, Color> = FoldMap::new();
  if let Some(ui) = colorscheme.get("ui")
    && let Some(ui_table) = ui.as_table()
  {
    for (key, val) in ui_table.iter() {
      if plain_colors.contains(key.as_str()) {
        if val.is_str() {
          let value = val.as_str().unwrap();
          let value = match palette.get(value) {
            Some(code) => *code,
            None => parse_code(value, "ui.", key)?,
          };
          let id = format!("ui.{}", key).to_compact_string();
          result.insert(id, value);
        } else {
          return err(key);
        }
      }
    }
  }

  Ok(result)
}

fn parse_syntax_highlights(
  colorscheme: &toml::Table,
  palette: &FoldMap<CompactString, Color>,
) -> TheResult<FoldMap<CompactString, Highlight>> {
  let err = |k: &str| {
    TheErr::LoadColorSchemeFailed(format!("syn.{}", k).to_compact_string())
  };

  let mut result: FoldMap<CompactString, Highlight> = FoldMap::new();
  if let Some(syn) = colorscheme.get("syn")
    && let Some(syn_table) = syn.as_table()
  {
    for (key, val) in syn_table.iter() {
      let id = format!("syn.{}", key).to_compact_string();
      if val.is_table() {
        let val_table = val.as_table().unwrap();

        let parse_color = |x| -> TheResult<Option<Color>> {
          match val_table.get(x) {
            Some(x) => {
              let x = x.as_str().ok_or(err(key))?;
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
          match val_table.get(x) {
            Some(x) => Ok(x.as_bool().ok_or(err(key))?),
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
        return Err(err(key));
      }
    }
  }

  Ok(result)
}

impl ColorScheme {
  pub fn from_empty(name: &str) -> Self {
    Self {
      name: name.to_compact_string(),
      foreground: DEFAULT_FOREGROUND_COLOR,
      background: DEFAULT_BACKGROUND_COLOR,
      syn: FoldMap::new(),
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
    let plain_colors = parse_plain_colors(&colorscheme, &palette)?;
    let syn = parse_syntax_highlights(&colorscheme, &palette)?;

    Ok(Self {
      name: name.to_compact_string(),
      foreground: *plain_colors
        .get(UI_FOREGROUND)
        .unwrap_or(&DEFAULT_FOREGROUND_COLOR),
      background: *plain_colors
        .get(UI_BACKGROUND)
        .unwrap_or(&DEFAULT_BACKGROUND_COLOR),
      syn,
    })
  }

  pub fn name(&self) -> &CompactString {
    &self.name
  }

  pub fn foreground(&self) -> &Color {
    &self.foreground
  }

  pub fn set_foreground(&mut self, value: Color) {
    self.foreground = value;
  }

  pub fn background(&self) -> &Color {
    &self.background
  }

  pub fn set_background(&mut self, value: Color) {
    self.background = value;
  }

  pub fn syn(&self) -> &FoldMap<CompactString, Highlight> {
    if cfg!(debug_assertions) {
      for k in self.syn.keys() {
        debug_assert!(k.starts_with("syn."));
      }
    }
    &self.syn
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

pub static DEFAULT_COLORSCHEME: Lazy<ColorScheme> = Lazy::new(|| {
  let config = toml::toml! {
    [syn]
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

    [ui]
    foreground = "white"
    background = "black"
  };
  ColorScheme::from_toml("default", config).unwrap()
});

impl ColorSchemeManager {
  pub fn new() -> Self {
    let mut highlights = FoldMap::new();
    highlights
      .insert("default".to_compact_string(), DEFAULT_COLORSCHEME.clone());
    Self { highlights }
  }

  pub fn is_empty(&self) -> bool {
    self.highlights.is_empty()
  }

  pub fn len(&self) -> usize {
    self.highlights.len()
  }

  pub fn get(&self, id: &str) -> Option<&ColorScheme> {
    self.highlights.get(id)
  }

  pub fn contains_key(&self, id: &str) -> bool {
    self.highlights.contains_key(id)
  }

  pub fn insert(
    &mut self,
    key: CompactString,
    value: ColorScheme,
  ) -> Option<ColorScheme> {
    self.highlights.insert(key, value)
  }

  pub fn remove(&mut self, id: &str) -> Option<ColorScheme> {
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
